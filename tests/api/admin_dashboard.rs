use crate::helpers;
use crate::helpers::assert_is_redirect_to;

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let app = helpers::spawn_app().await;

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
}
