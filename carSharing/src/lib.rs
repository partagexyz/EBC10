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
    price: Decimal,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct CarOwnerBadge {
    owner_number: u64,
    owner_name: String,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct UserBadge {
    user_number: u64,
    user_name: String,
    driving_license: String,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
struct RentalBadge {
    car_nft_id: NonFungibleGlobalId,
    user_badge_id: NonFungibleGlobalId,
    start_time: Instant,
    duration_in_hours: u32,
}

#[blueprint]
mod car_sharing {
    // define auth rules
    enable_method_auth! {
    roles {
        car_owner => updatable_by: [OWNER];
        user => updatable_by: [OWNER];
    },
        // decide which methods are public and which are restricted to certain roles
        methods {
            mint_car_owner_badge => restrict_to: [OWNER];
            mint_user_badge => restrict_to: [OWNER];
            mint_rental_badge => restrict_to: [OWNER];
            add_car => restrict_to: [car_owner];
            rent_car => restrict_to: [user];
            return_car => restrict_to: [user];
            list_available_cars => restrict_to: [user];
        }
    }
    struct carSharing {
        // The vault will hold car NFTs
        cars_vault: Vault,
        car_resource_manager: ResourceManager,
        // available_cars: BTreeMap<String, (Global<ResourceAddress>, NonFungibleId)>,
        car_owner_badge_resource_manager: ResourceManager,
        user_badge_resource_manager: ResourceManager,
        rental_badge_resource_manager: ResourceManager, // New resource for rental badges
    }

    impl carSharing {
        // Function to instantiate the CarSharing blueprint component with initial NFTs
        pub fn instantiate_car_sharing() -> (Global<carSharing>, Bucket) {
            // reserve an address for the component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(carSharing::blueprint_id());

            // create an Owner Badge
            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Car Sharing Owner Badge", locked;
                    }
                ))
                .divisibility(DIVISIBILITY_NONE)
                .mint_initial_supply(1)
                .into();

            // create a new Car Owner Badge resource manager
            let car_owner_badges_manager: ResourceManager =
                ResourceBuilder::new_integer_non_fungible::<CarOwnerBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "Car Owner Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(owner_badge.resource_address()));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // create a new User Badge resource manager
            let user_badges_manager: ResourceManager =
                ResourceBuilder::new_integer_non_fungible::<UserBadge>(OwnerRole::None)
                    .metadata(metadata!(
                        init {
                            "name" => "User Badge", locked;
                        }
                    ))
                    .mint_roles(mint_roles! {
                        minter => rule!(require(global_caller(component_address)));
                        minter_updater => rule!(deny_all);
                    })
                    .recall_roles(recall_roles! {
                        recaller => rule!(require(owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // create a new User Badge resource manager
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
                        recaller => rule!(require(owner_badge.resource_address(),));
                        recaller_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(owner_badge.resource_address()));
                        burner_updater => rule!(deny_all);
                    })
                    // starting with no initial supply means a resource manger is produced instead of a bucket
                    .create_with_no_initial_supply();

            // Create a new resource called "CarNFT" for non-fungible car tokens
            let cars_bucket: Bucket =
                ResourceBuilder::new_ruid_non_fungible::<Car>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "CarNFT".to_string(), locked;
                            "symbol" => "CAR".to_string(), locked;
                        }
                    })
                    .mint_roles(mint_roles! {
                        minter => rule!(require(car_owner_badges_manager.address()));
                        minter_updater => rule!(deny_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(require(car_owner_badges_manager.address()));
                        burner_updater => rule!(deny_all);
                    })
                    .mint_initial_supply(
                        // Define initial cars
                        [
                            Car {
                                make: "Tesla".to_string(),
                                model: "Model S".to_string(),
                                year: 2022,
                                vin: "5YJSA1E26MF1XXXXX".to_string(),
                                mileage: 15000,
                                location: "New York".to_string(),
                                price: dec!("100"),
                            },
                            Car {
                                make: "BMW".to_string(),
                                model: "i3".to_string(),
                                year: 2021,
                                vin: "WBY7Z4C57M7XXXXX".to_string(),
                                mileage: 18300,
                                location: "Lisboa".to_string(),
                                price: dec!("80"),
                            },
                            Car {
                                make: "Audi".to_string(),
                                model: "e-tron".to_string(),
                                year: 2023,
                                vin: "WA1LAAGE4LB0XXXXX".to_string(),
                                mileage: 20000,
                                location: "Barcelona".to_string(),
                                price: dec!("120"),
                            },
                            Car {
                                make: "Nissan".to_string(),
                                model: "Leaf".to_string(),
                                year: 2020,
                                vin: "1N4AZ1CP7LC3XXXXX".to_string(),
                                mileage: 23000,
                                location: "Paris".to_string(),
                                price: dec!("40"),
                            },
                            Car {
                                make: "Chevrolet".to_string(),
                                model: "Bolt".to_string(),
                                year: 2021,
                                vin: "1G1FY6S00M4XXXXX".to_string(),
                                mileage: 12000,
                                location: "London".to_string(),
                                price: dec!("50"),
                            },
                        ],
                    )
                    .into();

            let car_rs_manager: ResourceManager = cars_bucket.resource_manager();

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let car_sharing_impl: Global<carSharing> = Self {
                cars_vault: Vault::with_bucket(cars_bucket),
                car_resource_manager: car_rs_manager,
                car_owner_badge_resource_manager: car_owner_badges_manager,
                user_badge_resource_manager: user_badges_manager,
                rental_badge_resource_manager: rental_badges_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .globalize();
            (car_sharing_impl, owner_badge)
        }

        pub fn mint_car_owner_badge(&mut self, name: String, number: u64) -> Bucket {
            // mint and receive a new car owner badge. requires an owner badge
            let car_owner_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    CarOwnerBadge {
                        owner_number: number,
                        owner_name: name,
                    },
                );
            car_owner_badge_bucket
        }
        pub fn mint_user_badge(&mut self, name: String, number: u64) -> Bucket {
            // mint and receive a new car owner badge. requires an owner badge
            let user_badge_bucket: Bucket =
                self.car_owner_badge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(number),
                    UserBadge {
                        user_number: number,
                        user_name: name,
                        driving_license: "TODO".to_string(),
                    },
                );
            user_badge_bucket
        }

        pub fn mint_rental_badge(
            &mut self,
            car_nft_id: NonFungibleGlobalId,
            user_badge_id: NonFungibleGlobalId,
            duration_in_hours: u32,
        ) -> Bucket {
            // TODO: why not NonFungibleBucket

            let time: Instant = Clock::current_time(TimePrecisionV2::Second);
            // Create a badge that grants temporary access to a specific car
            self.rental_badge_resource_manager
                .mint_ruid_non_fungible(RentalBadge {
                    car_nft_id: car_nft_id,
                    user_badge_id: user_badge_id,
                    start_time: time,
                    duration_in_hours: duration_in_hours,
                })
        }

        // Method to add a new car to the platform
        pub fn add_car(&mut self, car: Car) {
            self.car_resource_manager.mint_ruid_non_fungible(car);
        }

        // Method to allow users to rent a car (retrieve a car NFT)
        pub fn rent_car(
            &mut self,
            car_nft_id: NonFungibleGlobalId,
            user_badge_id: NonFungibleGlobalId,
            duration_in_hours: u32,
        ) -> (NonFungibleBucket, Bucket) {
            let (resource_address, nft_local_id): (ResourceAddress, NonFungibleLocalId) =
                car_nft_id.clone().into_parts();
            assert_eq!(resource_address, self.car_resource_manager.address());
            let rented_car: NonFungibleBucket = self
                .cars_vault
                .as_non_fungible()
                .take_non_fungible(&nft_local_id);
            let rental_badge: Bucket =
                self.mint_rental_badge(car_nft_id, user_badge_id, duration_in_hours);
            (rented_car, rental_badge)
        }

        // Method to return a car
        pub fn return_car(&mut self, rented_car: Bucket, rental_bagde: Bucket) {
            self.cars_vault.put(rented_car);
            rental_bagde.burn();
        }

        // Method to list available cars
        pub fn list_available_cars(&self) -> indexmap::IndexSet<NonFungibleLocalId> {
            let available_cars_ids: indexmap::IndexSet<NonFungibleLocalId> = self
                .cars_vault
                .as_non_fungible()
                .non_fungible_local_ids(100);
            // info!(available_cars_ids.to_string());
            available_cars_ids
        }
    }
}
