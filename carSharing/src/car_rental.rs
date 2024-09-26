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
    start_time: Instant,
    duration_in_hours: u32,
}

#[blueprint]
mod car_rental {
    // define auth rules
    enable_method_auth! {
    roles {
        car_owner => updatable_by: [OWNER];
        user => updatable_by: [OWNER];
    },
        // decide which methods are public and which are restricted to certain roles
        methods {
            mint_rental_badge => restrict_to: [OWNER];
            withdraw_car_owner_vault => restrict_to: [car_owner];
            rent_car => restrict_to: [user];
            return_car => restrict_to: [user];
        }
    }
    struct CarRental {
        price_per_hour: Decimal,
        rental_badge_resource_manager: ResourceManager,
        car_owner_vault: Vault,
        fees_vault: Vault,
    }

    impl CarRental {
        // Function to instantiate the CarSharing blueprint component with initial NFTs
        // TODO: can it be protected so only the owner of a CarSharing can implement a CarRental
        pub fn instantiate_car_rental(
            car_owner_badge_address: ResourceAddress,
            user_badge_address: ResourceAddress,
            price_per_hour: Decimal,
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
            // TODO: It may be a better design to mint a single badge and update its (meta)data
            let rental_badges_manager: ResourceManager =
                ResourceBuilder::new_ruid_non_fungible::<RentalBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Rental Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(component_owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(component_owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let car_sharing_impl: Global<CarRental> = Self {
                price_per_hour: price_per_hour,
                rental_badge_resource_manager: rental_badges_manager,
                car_owner_vault: Vault::new(XRD),
                fees_vault: Vault::new(XRD),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                component_owner_badge.resource_address()
            ))))
            .roles(roles!(
                // TODO: use a NFT global ID instead of just an address
                car_owner => rule!(require(car_owner_badge_address));
                user => rule!(require(user_badge_address));))
            .with_address(address_reservation)
            .globalize();
            (car_sharing_impl, component_owner_badge)
        }

        pub fn mint_rental_badge(&mut self, duration_in_hours: u32) -> Bucket {
            let time: Instant = Clock::current_time(TimePrecisionV2::Second);
            // Create a badge that grants temporary access to a specific car
            self.rental_badge_resource_manager
                .mint_ruid_non_fungible(RentalBadge {
                    start_time: time,
                    duration_in_hours: duration_in_hours,
                })
        }

        // Withdraw the content of a CarOwner vaul
        pub fn withdraw_car_owner_vault(&mut self) -> Bucket {
            // TODO: make sure the car_owner role is protected for a single unique car owner and not anyone
            self.car_owner_vault.take_all()
        }

        // Method to allow users to rent a car (retrieve a car NFT)
        pub fn rent_car(&mut self, duration_in_hours: u32, payment_bucket: Bucket) -> Bucket {
            // TODO: implement logic to check if car is available
            let rental_price: Decimal = self.price_per_hour * duration_in_hours;
            assert!(
                payment_bucket.amount() >= rental_price,
                "You did not provided enough XRD for this rental."
            );
            let rental_badge: Bucket = self.mint_rental_badge(duration_in_hours);

            self.car_owner_vault.put(payment_bucket);
            rental_badge
        }

        // Method to return a car
        pub fn return_car(&mut self, rental_badge: Bucket) {
            // TODO: check if rental duration was respected
            rental_badge.burn();
        }
    }
}
