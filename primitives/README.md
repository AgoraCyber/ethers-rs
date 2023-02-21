# Ethereum core types

The crate defines the rust mapping of [`enthereum core types`](https://docs.soliditylang.org/en/v0.8.17/abi-spec.html?highlight=event#mapping-solidity-to-abi-types).

| abi                         | rust                                         |     Status |
| --------------------------- | ----------------------------------------     | ---------: |
| `uint<M>`/`int<M>`          | [`crate::IntM`](./src/primitives/int.rs)     |     support|
| `bytes<M>`                  | [`crate::BytesM`](./src/primitives/bytes.rs) |     support|
| `bytes`                     | [`crate::Bytes`](./src/primitives/bytes.rs)  |     support|
| `address`                   | [`crate::Bytes`](./src/primitives/address.rs)  |     support|
| bool                        | `bool`                                       |     support|
| `<type>[M]`                 | `[T;N]/&[T;N]`                               |     support|
| `<type>[]`                  | `Vec<T>/&[T]`                                |     support|
| `fixed<M>x<N>`              |                                              |  working on|
| `(T1,T2,...,Tn)`            | `(T1,T2,...,Tn)/struct A(T1,T2,...,Tn)/struct A { t1:T1,t2:T2,...,t3:Tn }`   |     support|

## With serde

This crate using [`serde`](https://serde.rs/) framework to handle contract abi serialize/deserilize. Therefore, it is easy to define the structure that supports the `Ethereum ABI`.

```rust

use ethers_types_rs::*;

#[derive(Debug,Serialize,Deserilize)]
struct Transfer {
    to: Address,
    amount: U256,
}
```

## EIP712 support

Again, thanks to the good framework design of [`serde`](https://serde.rs/).
this crate define a series of [`serializers`](src/serde/eip712) to support eip712 protocol.

```rust
let domain = json!({
    "name": "Ether Mail",
    "version": "1",
    "chainId": 1,
    "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
});

let domain: EIP712Domain = serde_json::from_value(domain).unwrap();
assert_eq!(
    eip712_hash_struct(&domain).unwrap().to_eth_hex(),
    "0xf2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f"
);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
    pub name: String,
    pub wallet: Address,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Mail {
    pub from: Person,
    pub to: Person,
    pub contents: String,
}

let json = json!({
  "from": {
    "name": "Cow",
    "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
  },
  "to": {
    "name": "Bob",
    "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
  },
  "contents": "Hello, Bob!"
});

let mail: Mail = serde_json::from_value(json).expect("parse domain");

assert_eq!(
    "Mail(Person from,Person to,string contents)Person(string name,address wallet)",
    eip712_encode_type(&mail).expect("generate e712 types")
);

assert_eq!(
    eip712_hash_struct(&mail.from).unwrap().to_eth_hex(),
    "0xfc71e5fa27ff56c350aa531bc129ebdf613b772b6604664f5d8dbe21b85eb0c8"
);

assert_eq!(
    eip712_hash_struct(&mail.to).unwrap().to_eth_hex(),
    "0xcd54f074a4af31b4411ff6a60c9719dbd559c221c8ac3492d9d872b041d703d1"
);

assert_eq!(
    eip712_hash_struct(&mail).unwrap().to_eth_hex(),
    "0xc52c0ee5d84264471806290a3f2c4cecfc5490626bf912d01f240d7a274b371e"
);

assert_eq!(
    eip712_encode(&domain, &mail).unwrap().to_eth_hex(),
    "0x1901f2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090fc52c0ee5d84264471806290a3f2c4cecfc5490626bf912d01f240d7a274b371e"
);

let expect_request: TypedData<Mail> =
    serde_json::from_str(include_str!("./eip712.json")).unwrap();

 assert_eq!(eip712_into_request(domain, mail).unwrap(), expect_request);
```
