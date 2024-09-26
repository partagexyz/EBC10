use scrypto::prelude::*;
#[derive(ScryptoSbor, NonFungibleData)]
pub struct Car {
    // Define what data will be stored for each car NFT
    make: String,
    model: String,
    year: u32,
    vin: String, // unique Vehicle Identification Number
    mileage: u32,
    location: String,
    price_per_hour: Decimal,
    owner_badge_id: NonFungibleLocalId,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct RentalBadge {
    #[mutable]
    start_time: Instant,
    #[mutable]
    duration_in_hours: u32,
}

#[blueprint]
mod car_rental {
    // define auth rules
    enable_method_auth! {
    roles {
        user => updatable_by: [OWNER];
    },
        methods {
            withdraw_car_owner_vault => PUBLIC;
            is_available => restrict_to: [user];
            rent_car => restrict_to: [user];
            return_car => restrict_to: [user];
        }
    }
    struct CarRental {
        price_per_hour: Decimal,
        rental_badge_vault: Vault,
        car_owner_vault: Vault,
        car_owner_global_id: NonFungibleGlobalId,
        fees_vault: Vault,
        fee: Decimal,
    }

    impl CarRental {
        // Function to instantiate the CarSharing blueprint component with initial NFTs
        // TODO: protect it so only the owner of a CarSharing can implement a CarRental
        pub fn instantiate_car_rental(
            car_owner_badge_global_id: NonFungibleGlobalId,
            user_badge_address: ResourceAddress,
            price_per_hour: Decimal,
            fee: Decimal,
        ) -> (Global<CarRental>, Bucket) {
            // reserve an address for the component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(CarRental::blueprint_id());

            // create an Owner Badge
            let component_owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Car Rental Component Owner Badge", locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // create a new User Badge resource manager
            let rental_badge_bucket: NonFungibleBucket =
                ResourceBuilder::new_ruid_non_fungible::<RentalBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Rental Badge", locked;
                        }
                    ))
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(component_owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(component_owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    .non_fungible_data_update_roles(non_fungible_data_update_roles!{
                        non_fungible_data_updater => rule!(require(component_owner_badge.resource_address()));
                        non_fungible_data_updater_updater => rule!(deny_all);
                    })
                    .mint_initial_supply([RentalBadge {
                        start_time: Clock::current_time(TimePrecisionV2::Second),
                        duration_in_hours: 0,
                    }]);

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let car_sharing_impl: Global<CarRental> = Self {
                price_per_hour: price_per_hour,
                rental_badge_vault: Vault::with_bucket(rental_badge_bucket.into()),
                car_owner_global_id: car_owner_badge_global_id,
                car_owner_vault: Vault::new(XRD),
                fees_vault: Vault::new(XRD),
                fee: fee,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                component_owner_badge.resource_address()
            ))))
            .roles(roles!(
                user => rule!(require(user_badge_address));
            ))
            .with_address(address_reservation)
            .globalize();
            (car_sharing_impl, component_owner_badge)
        }

        // Withdraw the content of a CarOwner vaul
        pub fn withdraw_car_owner_vault(&mut self, car_owner_proof: Proof) -> Bucket {
            let checked_proof: CheckedProof =
                car_owner_proof.check(self.car_owner_global_id.resource_address());
            assert_eq!(
                &checked_proof.as_non_fungible().non_fungible_local_id(),
                self.car_owner_global_id.local_id()
            );

            self.car_owner_vault.take_all()
        }

        pub fn is_available(&self) -> bool {
            !self.rental_badge_vault.is_empty()
        }

        // Method to allow users to rent a car (retrieve a car NFT)
        pub fn rent_car(&mut self, duration_in_hours: u32, mut payment_bucket: Bucket) -> Bucket {
            // TODO: add a deposit
            assert!(self.is_available(), "Car is not available to rent.");
            let rental_price: Decimal = self.price_per_hour * duration_in_hours + self.fee;
            assert!(
                payment_bucket.amount() >= rental_price,
                "You did not provided enough XRD for this rental."
            );
            let rental_badge: Bucket = self.rental_badge_vault.take(1);
            let rental_bag_rm: ResourceManager =
                ResourceManager::from_address(rental_badge.resource_address());

            rental_bag_rm.update_non_fungible_data(
                &rental_badge.as_non_fungible().non_fungible_local_id(),
                "start_time",
                Clock::current_time(TimePrecisionV2::Second),
            );
            rental_bag_rm.update_non_fungible_data(
                &rental_badge.as_non_fungible().non_fungible_local_id(),
                "duration",
                duration_in_hours,
            );
            self.fees_vault.put(payment_bucket.take(self.fee));
            self.car_owner_vault.put(payment_bucket);
            rental_badge
        }

        // Method to return a car
        pub fn return_car(&mut self, rental_badge: Bucket) {
            // TODO: check if rental duration was respected
            self.rental_badge_vault.put(rental_badge);
        }
    }
}
