use near_sdk::log;

use crate::*;

/// Private contract methods
impl FPOContract {
    pub fn assert_admin(&self) {
        log!("HIIII");
        assert_eq!(
            self.admin,
            env::predecessor_account_id(),
            "Only callable by admin {}",
            self.admin
        );
    }
}

/// Public contract methods
#[near_bindgen]
impl FPOContract {
    pub fn transfer_admin(&mut self, new_admin: AccountId) {
        self.assert_admin();
        self.admin = new_admin;
    }
}

/// Admin tests
#[cfg(test)]
mod tests {

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    use super::*;

    fn alice() -> AccountId {
        "alice.near".parse().unwrap()
    }
    fn bob() -> AccountId {
        "bob.near".parse().unwrap()
    }

    fn get_context(account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(account_id.clone())
            .signer_account_id("robert.testnet".parse().unwrap())
            .predecessor_account_id(account_id.clone());
        builder
    }

    #[test]
    fn transfer_admin() {
        // use alice
        let context = get_context(alice());
        testing_env!(context.build());

        // create fpo contract
        let mut fpo_contract = FPOContract::new(alice());

        // verify alice is admin
        assert_eq!(fpo_contract.admin, alice());

        // transfer admin to bob
        fpo_contract.transfer_admin(bob());

        // verify bob is admin
        assert_eq!(fpo_contract.admin, bob());
    }
}
