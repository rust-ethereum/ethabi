extern crate ethabi;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;

use_contract!(eip20, "Eip20", "../examples/eip20.json");

#[test]
fn it_works() {
	let c = kovan::Bridge::new();
	assert!(false);
}
