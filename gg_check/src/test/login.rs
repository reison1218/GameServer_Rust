use crate::check::login::{exchange_code_for_token, verify_google_id_token};

#[test]
fn test_login() {
    let res = exchange_code_for_token("4/0AVMBsJgRiDjNbLa4CCP_3cWZsd-az45kSBwP5u57-ksKLnh5WMFIsquq1xfxE0FiBhnn5w").unwrap();
    println!("server_token: {:?}", res);
    let id_token = res.id_token.as_ref().unwrap().as_str();
    let res = verify_google_id_token(id_token).unwrap();
    println!("{:?}", res);
}
