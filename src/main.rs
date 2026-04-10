use youtui_rs::ui::app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new().await?;
    app.run()?;
    Ok(())
}
