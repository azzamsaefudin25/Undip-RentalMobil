#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct RentalRequest {
    id: u64,
    requester: String,
    car_model: String,
    start_time: u64,
    end_time: u64,
    status: String,
}

impl Storable for RentalRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for RentalRequest {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, RentalRequest, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct RentalRequestPayload {
    requester: String,
    car_model: String,
    start_time: u64,
    end_time: u64,
}

#[ic_cdk::query]
fn get_rental_request(id: u64) -> Result<RentalRequest, Error> {
    match _get_rental_request(&id) {
        Some(request) => Ok(request),
        None => Err(Error::NotFound {
            msg: format!("a rental request with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn create_rental_request(request: RentalRequestPayload) -> Option<RentalRequest> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let request = RentalRequest {
        id,
        requester: request.requester,
        car_model: request.car_model,
        start_time: request.start_time,
        end_time: request.end_time,
        status: "pending".to_string(),
    };
    do_insert(&request);
    Some(request)
}

#[ic_cdk::update]
fn approve_rental_request(id: u64) -> Result<RentalRequest, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut request) => {
            request.status= "approved".to_string();
            do_insert(&request);
            Ok(request)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't approve a rental request with id={}. request not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn reject_rental_request(id: u64) -> Result<RentalRequest, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut request) => {
            request.status = "rejected".to_string();
            do_insert(&request);
            Ok(request)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't reject a rental request with id={}. request not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn return_car(id: u64) -> Result<RentalRequest, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(request) => Ok(request),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't return a car with id={}. request not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

fn _get_rental_request(id: &u64) -> Option<RentalRequest> {
    STORAGE.with(|service| service.borrow().get(id))
}

fn do_insert(request: &RentalRequest) {
    STORAGE.with(|service| service.borrow_mut().insert(request.id, request.clone()));
}

ic_cdk::export_candid!();

// #[macro_use]
// extern crate serde;
// use candid::{Decode, Encode};
// use ic_cdk::api::time;
// use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
// use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
// use std::{borrow::Cow, cell::RefCell};

// type Memory = VirtualMemory<DefaultMemoryImpl>;
// type IdCell = Cell<u64, Memory>;

// #[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
// struct InventoryItem {
//     id: u64,
//     name: String,
//     description: String,
//     price: u64,
//     quantity: u64,
// }

// // a trait that must be implemented for a struct that is stored in a stable struct
// impl Storable for InventoryItem {
//     fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
//         Cow::Owned(Encode!(self).unwrap())
//     }

//     fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
//         Decode!(bytes.as_ref(), Self).unwrap()
//     }
// }

// // another trait that must be implemented for a struct that is stored in a stable struct
// impl BoundedStorable for InventoryItem {
//     const MAX_SIZE: u32 = 1024;
//     const IS_FIXED_SIZE: bool = false;
// }

// thread_local! {
//     static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
//         MemoryManager::init(DefaultMemoryImpl::default())
//     );

//     static ID_COUNTER: RefCell<IdCell> = RefCell::new(
//         IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
//             .expect("Cannot create a counter")
//     );

//     static STORAGE: RefCell<StableBTreeMap<u64, InventoryItem, Memory>> =
//         RefCell::new(StableBTreeMap::init(
//             MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
//     ));
// }

// #[derive(candid::CandidType, Serialize, Deserialize, Default)]
// struct InventoryItemPayload {
//     name: String,
//     description: String,
//     price: u64,
//     quantity: u64,
// }

// #[ic_cdk::query]
// fn get_inventory_item(id: u64) -> Result<InventoryItem, Error> {
//     match _get_inventory_item(&id) {
//         Some(item) => Ok(item),
//         None => Err(Error::NotFound {
//             msg: format!("an inventory item with id={} not found", id),
//         }),
//     }
// }

// #[ic_cdk::update]
// fn add_inventory_item(item: InventoryItemPayload) -> Option<InventoryItem> {
//     let id = ID_COUNTER
//         .with(|counter| {
//             let current_value = *counter.borrow().get();
//             counter.borrow_mut().set(current_value + 1)
//         })
//         .expect("cannot increment id counter");
//     let item = InventoryItem {
//         id,
//         name: item.name,
//         description: item.description,
//         price: item.price,
//         quantity: item.quantity,
//     };
//     do_insert(&item);
//     Some(item)
// }

// #[ic_cdk::update]
// fn update_inventory_item(id: u64, payload: InventoryItemPayload) -> Result<InventoryItem, Error> {
//     match STORAGE.with(|service| service.borrow().get(&id)) {
//         Some(mut item) => {
//             item.name = payload.name;
//             item.description = payload.description;
//             item.price = payload.price;
//             item.quantity = payload.quantity;
//             do_insert(&item);
//             Ok(item)
//         }
//         None => Err(Error::NotFound {
//             msg: format!("couldn't update an inventory item with id={}. item not found", id),
//         }),
//     }
// }

// // helper method to perform insert.
// fn do_insert(item: &InventoryItem) {
//     STORAGE.with(|service| service.borrow_mut().insert(item.id, item.clone()));
// }

// #[ic_cdk::update]
// fn delete_inventory_item(id: u64) -> Result<InventoryItem, Error> {
//     match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
//         Some(item) => Ok(item),
//         None => Err(Error::NotFound {
//             msg: format!("couldn't delete an inventory item with id={}. item not found.", id),
//         }),
//     }
// }

// #[derive(candid::CandidType, Deserialize, Serialize)]
// enum Error {
//     NotFound { msg: String },
// }

// // a helper method to get an inventory item by id. used in get_inventory_item/update_inventory_item
// fn _get_inventory_item(id: &u64) -> Option<InventoryItem> {
//     STORAGE.with(|service| service.borrow().get(id))
// }

// // need this to generate candid
// ic_cdk::export_candid!();