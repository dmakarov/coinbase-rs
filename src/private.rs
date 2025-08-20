use std::thread;
use std::time::Duration;

use bigdecimal::BigDecimal;
use futures::stream::Stream;
use hyper::Uri;
use uritemplate::UriTemplate;
use uuid::Uuid;

use crate::{public::Public, request, CBError, DateTime, Result};

pub struct Private {
    _pub: Public,
    key: String,
    secret: String,
}

impl Private {
    pub fn new(uri: &str, key: &str, secret: &str) -> Self {
        Self {
            _pub: Public::new(uri),
            key: key.to_string(),
            secret: secret.to_string(),
        }
    }

    ///
    /// **List accounts**
    ///
    /// Lists current user’s accounts to which the authentication method has access to.
    ///
    /// https://docs.cdp.coinbase.com/api-reference/advanced-trade-api/rest-api/accounts/list-accounts
    ///
    pub async fn accounts(&self) -> Result<Vec<Account>> {
        let uri = UriTemplate::new("/api/v3/brokerage/accounts").build();
        let request = self.request(&uri);

        thread::sleep(Duration::from_millis(350));

        let request = request.clone().build();
        let request_future = self._pub.client.request(request);

        let response = request_future.await?;
        let body = hyper::body::to_bytes(response.into_body()).await?;

        match serde_json::from_slice::<Accounts>(&body) {
            Ok(body) => Ok(body.accounts),
            Err(e) => match serde_json::from_slice(&body) {
                Ok(coinbase_err) => Err(CBError::Coinbase(coinbase_err)),
                Err(_) => Err(CBError::Serde(e)),
            },
        }
    }

    ///
    /// **List transactions**
    ///
    /// Lists account’s transactions.
    ///
    /// https://developers.coinbase.com/api/v2#list-transactions
    ///
    pub fn transactions<'a>(
        &'a self,
        account_id: &Uuid,
    ) -> impl Stream<Item = Result<Vec<Transaction>>> + 'a {
        let limit = 100;
        let uri = UriTemplate::new("/v2/accounts/{account}/transactions{?query*}")
            .set("account", account_id.to_string())
            .set("query", &[("limit", limit.to_string().as_ref())])
            .build();
        let request = self.request(&uri);
        self._pub.get_stream(request)
    }

    ///
    /// **List addresses**
    ///
    /// Lists addresses for an account.
    ///
    /// https://docs.cloud.coinbase.com/sign-in-with-coinbase/docs/api-addresses#list-addresses
    ///
    pub fn list_addresses<'a>(
        &'a self,
        account_id: &Uuid,
    ) -> impl Stream<Item = Result<Vec<Address>>> + 'a {
        let uri = UriTemplate::new("/v2/accounts/{account}/addresses")
            .set("account", account_id.to_string())
            .build();
        let request = self.request(&uri);
        self._pub.get_stream(request)
    }

    pub async fn list_payment_methods(&self) -> Result<Vec<PaymentMethod>> {
        let uri = UriTemplate::new("/api/v3/brokerage/payment_methods").build();
        let request = self.request(&uri);

        thread::sleep(Duration::from_millis(350));

        let request = request.clone().build();
        let request_future = self._pub.client.request(request);

        let response = request_future.await?;
        let body = hyper::body::to_bytes(response.into_body()).await?;

        match serde_json::from_slice::<PaymentMethods>(&body) {
            Ok(body) => Ok(body.payment_methods),
            Err(e) => match serde_json::from_slice(&body) {
                Ok(coinbase_err) => Err(CBError::Coinbase(coinbase_err)),
                Err(_) => Err(CBError::Serde(e)),
            },
        }
    }

    pub async fn list_public_products(&self) -> Result<Vec<PublicProduct>> {
        let uri = UriTemplate::new("/api/v3/brokerage/market/products").build();
        let request = self.request(&uri);

        thread::sleep(Duration::from_millis(350));

        let request = request.clone().build();
        let request_future = self._pub.client.request(request);

        let response = request_future.await?;
        let body = hyper::body::to_bytes(response.into_body()).await?;

        match serde_json::from_slice::<PublicProducts>(&body) {
            Ok(body) => Ok(body.products),
            Err(e) => match serde_json::from_slice(&body) {
                Ok(coinbase_err) => Err(CBError::Coinbase(coinbase_err)),
                Err(_) => Err(CBError::Serde(e)),
            },
        }
    }

    pub async fn withdrawals(
        &self,
        account_id: &Uuid,
        amount: String,
        currency: String,
        payment_method: Uuid,
    ) -> Result<Transfer> {
        let uri = UriTemplate::new("/v2/accounts/{account}/withdrawals")
            .set("account", account_id.to_string())
            .build();
        let request = self.request(&uri);

        thread::sleep(Duration::from_millis(350));

        let body = match serde_json::to_vec(&Withdrawal {
            amount,
            currency,
            payment_method: payment_method.to_string(),
            commit: true,
        }) {
            Ok(body) => body,
            Err(e) => return Err(CBError::Serde(e)),
        };
        let request = request
            .clone()
            .method(http::Method::POST)
            .body(&body)
            .build();
        let request_future = self._pub.client.request(request);

        let response = request_future.await?;
        let body = hyper::body::to_bytes(response.into_body()).await?;

        match serde_json::from_slice::<Transfer>(&body) {
            Ok(body) => Ok(body),
            Err(e) => match serde_json::from_slice(&body) {
                Ok(coinbase_err) => Err(CBError::Coinbase(coinbase_err)),
                Err(_) => Err(CBError::Serde(e)),
            },
        }
    }

    fn request(&self, _uri: &str) -> request::Builder {
        let uri: Uri = (self._pub.uri.to_string() + _uri).parse().unwrap();
        request::Builder::new_with_auth(&self.key, &self.secret).uri(uri)
    }
}

#[derive(Deserialize, Debug)]
pub struct Value {
    pub value: String,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub uuid: String,
    pub name: String,
    pub currency: String,
    pub available_balance: Value,
    pub default: bool,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub r#type: String,
    pub ready: bool,
    pub hold: Value,
    pub retail_portfolio_id: String,
    pub platform: String,
}

#[derive(Deserialize, Debug)]
pub struct Accounts {
    pub has_next: bool,
    pub accounts: Vec<Account>,
    pub cursor: String,
    pub size: usize,
}

#[derive(Deserialize, Debug)]
pub struct Balance {
    pub amount: BigDecimal,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    pub id: String,
    pub address: String,
    pub name: Option<String>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    pub network: String,
    pub resource: String,
    pub resource_path: String,
}

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub id: Uuid,

    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,

    pub r#type: String,
    pub resource: String,
    pub resource_path: String,
    pub status: String,
    pub amount: Balance,
    pub native_amount: Balance,
    pub instant_exchange: bool,
    pub network: Option<Network>,
    pub from: Option<From>,
    pub details: TransactionDetails,
}

#[derive(Deserialize, Debug)]
pub struct Network {
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct From {
    pub id: Option<Uuid>,
    pub resource: String,
    pub resource_path: Option<String>,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionDetails {
    pub title: String,
    pub subtitle: String,
}

#[derive(Deserialize, Debug)]
pub struct Currency {
    pub code: String,
    pub name: String,
    pub exponent: usize,
    pub r#type: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub enum Order {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub ending_before: Option<DateTime>,
    pub starting_after: Option<DateTime>,
    pub previous_ending_before: Option<String>,
    pub next_starting_after: Option<String>,
    pub limit: usize,
    pub order: Order,
    pub previous_uri: Option<String>,
    pub next_uri: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PaymentMethod {
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub currency: String,
    pub verified: bool,
    pub allow_buy: bool,
    pub allow_sell: bool,
    pub allow_deposit: bool,
    pub allow_withdraw: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Deserialize, Debug)]
pub struct PaymentMethods {
    pub payment_methods: Vec<PaymentMethod>,
}

#[derive(Deserialize, Debug)]
pub struct Maintenance {
    pub start_time: String,
    pub end_time: String,
}

#[derive(Deserialize, Debug)]
pub struct FcmTradingSessionDetails {
    pub is_session_open: bool,
    pub open_time: String,
    pub close_time: String,
    pub session_state: String,
    pub after_hours_order_entry_disabled: bool,
    pub closed_reason: String,
    pub maintenance: Maintenance,
}

#[derive(Deserialize, Debug)]
pub struct PerpetualDetails {
    pub open_interest: String,
    pub funding_rate: String,
    pub funding_time: String,
    pub max_leverage: String,
    pub base_asset_uuid: String,
    pub underlying_type: String,
}

#[derive(Deserialize, Debug)]
pub struct FutureProductDetails {
    pub venue: String,
    pub contract_code: String,
    pub contract_expiry: String,
    pub contract_size: String,
    pub contract_root_unit: String,
    pub group_description: String,
    pub contract_expiry_timezone: String,
    pub group_short_description: String,
    pub risk_managed_by: String,
    pub contract_expiry_type: String,
    pub perpetual_details: PerpetualDetails,
    pub contract_display_name: String,
    pub time_to_expiry_ms: String,
    pub non_crypto: bool,
    pub contract_expiry_name: String,
    pub twenty_four_by_seven: bool,
    pub funding_interval: String,
    pub open_interest: String,
    pub funding_rate: String,
    pub funding_time: String,
}

#[derive(Deserialize, Debug)]
pub struct PublicProduct {
    pub product_id: String,
    pub price: String,
    pub price_percentage_change_24h: String,
    pub volume_24h: String,
    pub volume_percentage_change_24h: String,
    pub base_increment: String,
    pub quote_increment: String,
    pub quote_min_size: String,
    pub quote_max_size: String,
    pub base_min_size: String,
    pub base_max_size: String,
    pub base_name: String,
    pub quote_name: String,
    pub watched: bool,
    pub is_disabled: bool,
    pub new: bool,
    pub status: String,
    pub cancel_only: bool,
    pub limit_only: bool,
    pub post_only: bool,
    pub trading_disabled: bool,
    pub action_mode: Option<bool>,
    pub base_display_symbol: String,
    pub quote_display_symbol: String,
    pub product_type: String,
    pub quote_currency_id: String,
    pub base_currency_id: String,
    pub fcm_trading_session_details: Option<FcmTradingSessionDetails>,
    pub mid_market_price: String,
    pub alias: String,
    pub alias_to: Vec<String>,
    pub view_only: bool,
    pub price_increment: String,
    pub display_name: String,
    pub product_venue: String,
    pub approximate_quote_24h_volume: String,
    pub new_at: String,
    pub future_product_details: Option<FutureProductDetails>,
}

#[derive(Deserialize, Debug)]
pub struct PublicProducts {
    pub products: Vec<PublicProduct>,
    pub num_products: usize,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Transfer {
    pub user_entered_amount: Amount,
    pub amount: Amount,
    pub total: Amount,
    pub subtotal: Amount,
    pub idem: String,
    pub committed: bool,
    pub id: String,
    pub instant: bool,
    pub source: Source,
    pub target: Target,
    pub payout_at: DateTime,
    pub status: String,
    pub user_reference: String,
    pub r#type: String,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    pub user_warnings: Vec<String>,
    pub fees: Vec<String>,
    pub total_fee: Fee,
    pub cancellation_reason: Option<String>,
    pub hold_days: usize,
    pub nextStep: Option<String>,
    pub checkout_url: String,
    pub requires_completion_step: bool,
}

#[derive(Deserialize, Debug)]
pub struct Amount {
    pub value: String,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct Source {
    pub r#type: String,
    pub network: String,
    pub payment_method_id: String,
    pub ledger_account: LedgerAccount,
}

#[derive(Deserialize, Debug)]
pub struct Target {
    pub r#type: String,
    pub network: String,
    pub payment_method_id: String,
    pub external_payment_method: ExternalPaymentMethod,
}

#[derive(Deserialize, Debug)]
pub struct Fee {
    pub title: String,
    pub description: String,
    pub amount: Amount,
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
pub struct LedgerAccount {
    pub account_id: String,
    pub currency: String,
    pub owner: Owner,
}

#[derive(Deserialize, Debug)]
pub struct ExternalPaymentMethod {
    pub payment_method_id: String,
}

#[derive(Deserialize, Debug)]
pub struct Owner {
    pub id: String,
    pub uuid: String,
    pub user_uuid: String,
    pub r#type: String,
}

#[derive(Serialize, Debug)]
pub struct Withdrawal {
    pub amount: String,
    pub currency: String,
    pub payment_method: String,
    pub commit: bool,
}

#[test]
fn test_pagination_deserialize() {
    let input = r##"
{
    "ending_before": null,
    "starting_after": null,
    "previous_ending_before": null,
    "next_starting_after": "d16ec1ba-b3f7-5d6a-a9c8-817930030324",
    "limit": 25,
    "order": "desc",
    "previous_uri": null,
    "next_uri": "/v2/accounts?starting_after=d16ec1ba-b3f7-5d6a-a9c8-817930030324"
}"##;
    let pagination: Pagination = serde_json::from_slice(input.as_bytes()).unwrap();
    assert_eq!(25, pagination.limit);
    assert_eq!(Order::Descending, pagination.order);
}

#[test]
fn test_account_deserialize() {
    let input = r##"[
  {
    "uuid": "8bfc20d7-f7c6-4422-bf07-8243ca4169fe",
    "name": "BTC Wallet",
    "currency": "BTC",
    "available_balance": {
      "value": "1.23",
      "currency": "BTC"
    },
    "default": false,
    "active": true,
    "created_at": "2021-05-31T09:59:59.000Z",
    "updated_at": "2021-05-31T09:59:59.000Z",
    "deleted_at": "2021-05-31T09:59:59.000Z",
    "type": "FIAT",
    "ready": true,
    "hold": {
      "value": "1.23",
      "currency": "BTC"
    },
    "retail_portfolio_id": "b87a2d3f-8a1e-49b3-a4ea-402d8c389aca",
    "platform": "ACCOUNT_PLATFORM_CONSUMER"
  }
]"##;

    let accounts: Vec<Account> = serde_json::from_slice(input.as_bytes()).unwrap();
    assert_eq!(accounts.len(), 1);
}

#[test]
fn test_transactions_deserialize() {
    let input = r#"[
{
  "id": "9dd482e4-d8ce-46f7-a261-281843bd2855",
  "type": "send",
  "status": "completed",
  "amount": {
    "amount": "-0.00100000",
    "currency": "BTC"
  },
  "native_amount": {
    "amount": "-0.01",
    "currency": "USD"
  },
  "description": null,
  "created_at": "2015-03-11T13:13:35-07:00",
  "updated_at": "2015-03-26T15:55:43-07:00",
  "resource": "transaction",
  "resource_path": "/v2/accounts/af6fd33a-e20c-494a-b3f6-f91d204af4b7/transactions/9dd482e4-d8ce-46f7-a261-281843bd2855",
  "network": {
    "status": "off_blockchain",
    "name": "bitcoin"
  },
  "to": {
    "id": "2dbc3cfb-ed1e-4c10-aedb-aeb1693e01e7",
    "resource": "user",
    "resource_path": "/v2/users/2dbc3cfb-ed1e-4c10-aedb-aeb1693e01e7"
  },
  "instant_exchange": false,
  "details": {
    "title": "Sent bitcoin",
    "subtitle": "to User 2"
  }
},
{
  "id": "c1c413d1-acf8-4fcb-a8ed-4e2e4820c6f0",
  "type": "buy",
  "status": "pending",
  "amount": {
    "amount": "1.00000000",
    "currency": "BTC"
  },
  "native_amount": {
    "amount": "10.00",
    "currency": "USD"
  },
  "description": null,
  "created_at": "2015-03-26T13:42:00-07:00",
  "updated_at": "2015-03-26T15:55:45-07:00",
  "resource": "transaction",
  "resource_path": "/v2/accounts/af6fd33a-e20c-494a-b3f6-f91d204af4b7/transactions/c1c413d1-acf8-4fcb-a8ed-4e2e4820c6f0",
  "buy": {
    "id": "ae7df6e7-fef1-441d-a6f3-e4661ca6f39a",
    "resource": "buy",
    "resource_path": "/v2/accounts/af6fd33a-e20c-494a-b3f6-f91d204af4b7/buys/ae7df6e7-fef1-441d-a6f3-e4661ca6f39a"
  },
  "instant_exchange": false,
  "details": {
    "title": "Bought bitcoin",
    "subtitle": "using Capital One Bank"
  }
}
]"#;
    let transactions: Vec<Transaction> = serde_json::from_slice(input.as_bytes()).unwrap();
    assert_eq!(transactions.len(), 2);
}
