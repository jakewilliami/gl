use crate::repo::discover_repository;
use gix::remote::Direction;

pub fn get_remote_origin_url() {
    if let Some(origin_url) = remote_origin_url() {
        println!("{origin_url}")
    }
}

// You can use either of the following:
//     git remote get-url origin
//     git config --get remote.origin.url
//
// <https://stackoverflow.com/a/32991784>
// <https://stackoverflow.com/a/4089452>
pub fn remote_origin_url() -> Option<String> {
    let repo = discover_repository()?;
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url(Direction::Fetch);
    url.map(|u| u.to_bstring().to_string())
}
