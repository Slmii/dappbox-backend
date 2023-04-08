use ic_cdk::export::Principal;

pub fn whitelist() -> Vec<Principal> {
	[
		Principal::from_text("ruphi-egejm-cyi7p-bp5p5-hqlah-6dolh-2lf4m-jir5c-yl7em-6z4ct-iae".to_string()).unwrap(),
		Principal::from_text("mgbww-qhkta-bxkf3-q3dpx-x2ula-ile6u-xvxxs-u2n66-2zm76-uavff-yqe".to_string()).unwrap(),
		Principal::from_text("u7afm-2yaaa-aaaao-ab53a-cai".to_string()).unwrap(),
		Principal::from_text("zukw5-kqaaa-aaaao-acaxa-cai".to_string()).unwrap(),
		Principal::from_text("fqd3w-cqaaa-aaaao-aaxyq-cai".to_string()).unwrap(),
	].to_vec()
}
