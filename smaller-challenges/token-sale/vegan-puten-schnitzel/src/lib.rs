use scrypto::prelude::*;
 
blueprint! {
   struct TokenSale {
       vps_tokens_vault: Vault,
       xrd_tokens_vault: Vault,
       price_per_token: Decimal,
   }
 
   impl TokenSale {
       pub fn new(price_per_token: Decimal) -> (ComponentAddress, Bucket) {
           let seller_badge: Bucket = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_NONE)                                    
                                    .metadata("name", "VPS seller badge")
                                    .metadata("symbol", "VPS_SELLER")
                                    .restrict_withdraw(rule!(deny_all), LOCKED)
                                    .initial_supply(1);
            let access_rules: AccessRules = AccessRules::new()     
                                    .method("change_price", rule!(require(seller_badge.resource_address())))
                                    .method("withdraw_funds", rule!(require(seller_badge.resource_address())))                                    
                                    .default(rule!(allow_all));

            let tokens: Bucket = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_MAXIMUM)
                                    // .divisibility(18)
                                    .metadata("name", "VEGAN-PUTZEN-SCHNITZEL")
                                    .metadata("symbol", "VPS")
                                    .metadata("team-member-1-ticket-number", "4025901289")
                                    .metadata("team-member-2-ticket-number", "4096586149")
                                    .metadata("team-member-3-ticket-number", "4146677539")
                                    .metadata("team-member-4-ticket-number", "4115553269")
                                    .initial_supply(100000);

            let component_address: ComponentAddress = Self {
                vps_tokens_vault: Vault::with_bucket(tokens),
                xrd_tokens_vault: Vault::new(RADIX_TOKEN),
                price_per_token: price_per_token,
            }
            .instantiate()
            .add_access_check(access_rules)
            .globalize();

            (component_address, seller_badge)
       }
 
       pub fn buy(&mut self, funds: Bucket) -> Bucket {
           let purchase_amount: Decimal = funds.amount() / self.price_per_token;
           self.xrd_tokens_vault.put(funds);
           self.vps_tokens_vault.take(purchase_amount)
       }
 
       pub fn withdraw_funds(&mut self, amount: Decimal) -> Bucket {
           self.xrd_tokens_vault.take(amount)
       }
 
       pub fn change_price(&mut self, price: Decimal) {
           self.price_per_token = price;
       }
   }
}