#[derive(serde::Serialize, serde::Deserialize)]
pub struct Region {
    pub id: String,
    pub x: f64,      // 0-1 relative to viewer width
    pub y: f64,      // 0-1 relative to viewer height
    pub width: f64,  // 0-1
    pub height: f64, // 0-1
    pub label: String,
    pub color: String,
    pub page_index: u32,
    pub start: Option<f64>, // seconds
}
