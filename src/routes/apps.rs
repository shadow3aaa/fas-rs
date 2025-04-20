use rocket::{get, serde::json::Json};

#[get("/api/apps")]
pub async fn get_installed_apps() -> Json<Vec<AppInfo>> {
    let packages = android::package_manager::get_installed_packages().await?;
    Json(packages
        .into_iter()
        .map(|pkg| AppInfo {
            name: pkg.app_name,
            package_name: pkg.package_name,
        })
        .collect())
}

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub name: String,
    pub package_name: String,
}