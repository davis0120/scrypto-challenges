use scrypto::prelude::*;

blueprint! {
    struct TokenSale {
        useful_tokens_vault: Vault,
        rxd_token_vault: Vault,
        price_per_token: Decimal

    }

    impl TokenSale {
        pub fn new(price_per_token: Decimal) -> (ComponentAddress, Bucket) {
            let bucket : Bucket = ResourceBuilder::new_fungible()
            .metadata("name", "AAB")
            .metadata("team-member-1-ticket-number", "#4069206879")
            .metadata("team-member-2-ticket-number", "#4069505699")
            .metadata("team-member-3-ticket-number", "#4065895799")
            .divisibility(DIVISIBILITY_MAXIMUM)
            .initial_supply(100000);

            let seller_badge : Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .metadata("name", "Seller badge")
                .metadata("symbol", "SELLER")
                .initial_supply(1);

            let acces_rules: AccessRules = AccessRules::new()
                .method("withdraw_funds", rule!(require(seller_badge.resource_address())))
                .method("change_price", rule!(require(seller_badge.resource_address())))
                .default(rule!(allow_all));


            let component_address : ComponentAddress = Self {
                useful_tokens_vault: Vault::with_bucket(bucket),
                rxd_token_vault: Vault::new(RADIX_TOKEN),
                price_per_token: price_per_token
            }
            .instantiate()
            .add_access_check(acces_rules)
            .globalize();

            (component_address, seller_badge)
        }

        pub fn buy(&mut self, funds: Bucket) -> Bucket {
            let purchase_amount: Decimal  = funds.amount() / self.price_per_token;
            self.rxd_token_vault.put(funds);
            self.useful_tokens_vault.take(purchase_amount)
        }

        pub fn withdraw_funds(&mut self, amount: Decimal) -> Bucket {
            self.rxd_token_vault.take(amount)
        }

        pub fn change_price(&mut self, price: Decimal) {
            self.price_per_token = price;
        }
    }
}
