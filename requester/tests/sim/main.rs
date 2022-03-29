use near_fpo::FPOContractContract;
pub use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use requester::RequesterContract;
use serde_json::json;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FPO_BYTES => "../res/near_fpo.wasm",
    REQUESTER_BYTES => "../res/requester.wasm"
}

pub const DEFAULT_GAS: u64 = 300_000_000_000_000;

fn init() -> (
    UserAccount,
    ContractAccount<FPOContractContract>,
    ContractAccount<RequesterContract>,
) {
    let root = init_simulator(None);
    // Deploy the compiled Wasm bytes
    let fpo: ContractAccount<FPOContractContract> = deploy! {
        contract: FPOContractContract,
        contract_id: "nearfpo".to_string(),
        bytes: &FPO_BYTES,
        signer_account: root
    };
    // Deploy the compiled Wasm bytes
    let requester: ContractAccount<RequesterContract> = deploy! {
        contract: RequesterContract,
        contract_id: "requester",
        bytes: &REQUESTER_BYTES,
        signer_account: root
    };

    (root, fpo, requester)
}

#[test]
fn simulate_get_price() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));
    call!(provider1, fpo.new()).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    call!(provider2, requester.new(fpo.account_id())).assert_success();

    let outcome = call!(
        provider2,
        requester.get_price("ETH/USD".to_string(), provider1.account_id())
    );
    match &outcome.promise_results()[2] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value(), "2000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_prices() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    call!(
        provider2,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("ETH/USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("ETH/USD".to_string(), provider2.account_id())
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    let outcome = call!(
        provider2,
        requester.get_prices(
            vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()]
        )
    );
    // // println!("{:?}", outcome.promise_results());
    match &outcome.promise_results()[2] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value(), json!(vec!["2000", "4000"]));
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_agg_avg() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    call!(
        provider2,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("ETH/USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("ETH/USD".to_string(), provider2.account_id())
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    let outcome = call!(
        provider2,
        requester.aggregate_avg(
            vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()],
            0
        )
    );
    // // println!("{:?}", outcome.promise_results());
    match &outcome.promise_results()[2] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value(), "3000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_agg_median() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    call!(
        provider2,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("ETH/USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("ETH/USD".to_string(), provider2.account_id())
    );
    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    let outcome = call!(
        provider2,
        requester.aggregate_median(
            vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()],
            0
        )
    );
    // // println!("{:?}", outcome.promise_results());
    match &outcome.promise_results()[2] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value(), "3000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_price_call() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    let outcome = call!(
        user,
        fpo.get_price_call(
            "ETH/USD".to_string(),
            provider1.account_id(),
            requester.account_id()
        )
    );

    // // println!("{:?}", outcome.promise_results());

    let fetched_entry = call!(
        user,
        requester.get_pair(provider1.account_id(), "ETH/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "2000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_prices_call() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    // create a price pair, check if it exists, and get the value
    call!(
        provider2,
        fpo.create_pair("BTC/USD".to_string(), 8, U128(45000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("BTC/USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("BTC/USD".to_string(), provider2.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"45000".to_string()
    );

    let outcome = call!(
        user,
        fpo.get_prices_call(
            vec!["ETH/USD".to_string(), "BTC/USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()],
            requester.account_id()
        )
    );

    // // println!("{:?}", outcome.promise_results());

    let fetched_entry = call!(
        user,
        requester.get_pair(provider1.account_id(), "ETH/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "2000");
        }
        None => println!("Retrieved Nothing"),
    }

    let fetched_entry = call!(
        user,
        requester.get_pair(provider2.account_id(), "BTC/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "45000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_get_prices_call2() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    // create a price pair, check if it exists, and get the value
    call!(
        provider2,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(4000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("ETH/USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("ETH/USD".to_string(), provider2.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    let outcome = call!(
        user,
        fpo.get_prices_call(
            vec!["ETH/USD".to_string(), "ETH/USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()],
            requester.account_id()
        )
    );

    // println!("{:?}", outcome.promise_results());

    let fetched_entry = call!(
        user,
        requester.get_pair(provider1.account_id(), "ETH/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "2000"); // why wrong??
        }
        None => println!("Retrieved Nothing"),
    }

    let fetched_entry = call!(
        user,
        requester.get_pair(provider2.account_id(), "ETH/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "4000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_aggregate_avg_call() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    // create a price pair, check if it exists, and get the value
    call!(
        provider2,
        fpo.create_pair("ETH / USD".to_string(), 8, U128(4000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("ETH / USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("ETH / USD".to_string(), provider2.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    let outcome = call!(
        user,
        fpo.aggregate_avg_call(
            vec!["ETH/USD".to_string(), "ETH / USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()],
            0,
            requester.account_id()
        )
    );

    // // println!("{:?}", outcome.promise_results());

    let fetched_entry = call!(
        user,
        requester.get_pair(provider1.account_id(), "ETH/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "3000");
        }
        None => println!("Retrieved Nothing"),
    }

    let fetched_entry = call!(
        user,
        requester.get_pair(provider2.account_id(), "ETH / USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "3000");
        }
        None => println!("Retrieved Nothing"),
    }
}

#[test]
fn simulate_aggregate_median_call() {
    let (root, fpo, requester) = init();

    let provider1 = root.create_user("provider1".parse().unwrap(), to_yocto("1000000"));
    let provider2 = root.create_user("provider2".parse().unwrap(), to_yocto("1000000"));

    let user = root.create_user("user".parse().unwrap(), to_yocto("1000000"));

    call!(root, fpo.new()).assert_success();
    call!(root, requester.new(fpo.account_id())).assert_success();

    // create a price pair, check if it exists, and get the value
    call!(
        provider1,
        fpo.create_pair("ETH/USD".to_string(), 8, U128(2000))
    )
    .assert_success();
    call!(
        provider1,
        fpo.pair_exists("ETH/USD".to_string(), provider1.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider1,
        fpo.get_entry("ETH/USD".to_string(), provider1.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"2000".to_string()
    );

    // create a price pair, check if it exists, and get the value
    call!(
        provider2,
        fpo.create_pair("ETH / USD".to_string(), 8, U128(4000))
    )
    .assert_success();
    call!(
        provider2,
        fpo.pair_exists("ETH / USD".to_string(), provider2.account_id())
    )
    .assert_success();
    let price_entry = call!(
        provider2,
        fpo.get_entry("ETH / USD".to_string(), provider2.account_id())
    );

    debug_assert_eq!(
        &price_entry.unwrap_json_value()["price"].to_owned(),
        &"4000".to_string()
    );

    let outcome = call!(
        user,
        fpo.aggregate_median_call(
            vec!["ETH/USD".to_string(), "ETH / USD".to_string()],
            vec![provider1.account_id(), provider2.account_id()],
            0,
            requester.account_id()
        )
    );

    // // println!("{:?}", outcome.promise_results());

    let fetched_entry = call!(
        user,
        requester.get_pair(provider1.account_id(), "ETH/USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "3000");
        }
        None => println!("Retrieved Nothing"),
    }

    let fetched_entry = call!(
        user,
        requester.get_pair(provider2.account_id(), "ETH / USD".to_string())
    );
    // println!("res: {:?}", fetched_entry);

    match &fetched_entry.promise_results()[1] {
        Some(res) => {
            // println!("Retrieved Value: {:?}", res.unwrap_json_value());
            assert_eq!(res.unwrap_json_value()["price"], "3000");
        }
        None => println!("Retrieved Nothing"),
    }
}
