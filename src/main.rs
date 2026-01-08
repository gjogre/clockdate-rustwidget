mod config;
mod font;

use chrono::Local;
use config::Config;
use glib::ControlFlow;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use pangocairo::functions::{create_layout, show_layout};
use std::rc::Rc;
use std::cell::RefCell;

const APP_ID: &str = "com.clockdate";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}

fn build_ui(app: &Application) {
    // Load configuration
    let config = Rc::new(Config::load_or_default());

    // Load fonts
    let (font_time, font_date) = font::load_embedded_figlet_fonts()
        .expect("Failed to load embedded fonts");
    let font_time = Rc::new(font_time);
    let font_date = Rc::new(font_date);

    // Load CSS for transparency
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        "window, box, drawing { background-color: rgba(0, 0, 0, 0); }"
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Create window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Clock & Date")
        .build();

    // Initialize layer shell
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);

    // Set the monitor from config
    if let Some(display) = gtk4::gdk::Display::default() {
        // Get list of monitors
        let monitors = display.monitors();
        let n_monitors = monitors.n_items();

        // Find the monitor by name
        for i in 0..n_monitors {
            if let Some(obj) = monitors.item(i) {
                if let Ok(monitor) = obj.downcast::<gtk4::gdk::Monitor>() {
                    if let Some(connector) = monitor.connector() {
                        if connector.as_str() == config.window.monitor {
                            LayerShell::set_monitor(&window, &monitor);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Anchor to top-right corner
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);

    // Set margins from config
    window.set_margin(Edge::Top, config.window.margin_top);
    window.set_margin(Edge::Right, config.window.margin_right);

    // Set keyboard interactivity to none
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    // Set exclusive zone to -1 (no exclusive zone)
    window.set_exclusive_zone(-1);

    // Set size
    window.set_default_size(config.window.width, config.window.height);

    // Create drawing area
    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(config.window.width);
    drawing_area.set_content_height(config.window.height);

    // Shared state for rendering
    let time_text = Rc::new(RefCell::new(String::new()));
    let date_text = Rc::new(RefCell::new(String::new()));

    // Set up draw function
    let config_clone = Rc::clone(&config);
    let time_text_clone = Rc::clone(&time_text);
    let date_text_clone = Rc::clone(&date_text);

    drawing_area.set_draw_func(move |_area, cr, width, _height| {
        // Clear with full transparency
        cr.save().ok();
        cr.set_operator(cairo::Operator::Clear);
        cr.paint().ok();
        cr.restore().ok();

        // Set source to transparent
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);

        // Parse colors
        let time_color = config::parse_color(&config_clone.colors.time);
        let date_color = config::parse_color(&config_clone.colors.date);

        // Convert ratatui color to RGB
        let (tr, tg, tb) = color_to_rgb(time_color);
        let (dr, dg, db) = color_to_rgb(date_color);

        // Get figlet text
        let time_figlet = time_text_clone.borrow();
        let date_figlet = date_text_clone.borrow();

        // Create pango layout for time
        let layout_time = create_layout(cr);
        layout_time.set_markup(&format!(
            "<span font_family='monospace' size='{}' foreground='#{:02x}{:02x}{:02x}'>{}</span>",
            config_clone.fonts.time_size * 1024,
            tr, tg, tb,
            glib::markup_escape_text(&time_figlet)
        ));

        // Create pango layout for date
        let layout_date = create_layout(cr);
        layout_date.set_markup(&format!(
            "<span font_family='monospace' size='{}' foreground='#{:02x}{:02x}{:02x}'>{}</span>",
            config_clone.fonts.date_size * 1024,
            dr, dg, db,
            glib::markup_escape_text(&date_figlet)
        ));

        // Get actual ink extents (visual bounds)
        let (time_ink, _) = layout_time.extents();
        let (date_ink, _) = layout_date.extents();

        // Convert from Pango units to pixels (1024 units = 1 pixel)
        let time_width_px = time_ink.width() / pango::SCALE;
        let time_height_px = time_ink.height() / pango::SCALE;
        let date_width_px = date_ink.width() / pango::SCALE;

        // Draw time at top
        let time_x = (width - time_width_px) / 2;
        let time_y = 0;
        cr.move_to(time_x as f64, time_y as f64);
        show_layout(cr, &layout_time);

        // Draw date with configured offset (typically negative to overlap and compensate for figlet empty lines)
        let date_x = (width - date_width_px) / 2;
        let date_y = time_y + time_height_px + config_clone.window.date_offset;
        cr.move_to(date_x as f64, date_y as f64);
        show_layout(cr, &layout_date);
    });

    window.set_child(Some(&drawing_area));

    // Update time every 250ms
    let drawing_area_clone = drawing_area.clone();
    let time_text_update = Rc::clone(&time_text);
    let date_text_update = Rc::clone(&date_text);

    glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
        let now = Local::now();
        let time_str = now.format("%H:%M").to_string();
        let date_str = now.format("%d.%m.%Y").to_string();

        let time_figlet = font::render_figlet_text(&font_time, &time_str);
        let date_figlet = font::render_figlet_text(&font_date, &date_str);

        *time_text_update.borrow_mut() = time_figlet.to_string();
        *date_text_update.borrow_mut() = date_figlet.to_string();

        drawing_area_clone.queue_draw();
        ControlFlow::Continue
    });

    // **CRITICAL: Set empty input region for click-through and enable transparency**
    window.connect_realize(|window| {
        if let Some(surface) = window.surface() {
            // Create empty region (no rectangles added) for click-through
            let region = cairo::Region::create();
            surface.set_input_region(&region);

            // Enable transparency
            surface.set_opaque_region(None);
        }
    });

    window.present();
}

fn color_to_rgb(color: ratatui::style::Color) -> (u8, u8, u8) {
    use ratatui::style::Color;
    match color {
        Color::Black => (0, 0, 0),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Yellow => (255, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::Gray => (128, 128, 128),
        Color::DarkGray => (64, 64, 64),
        Color::LightRed => (255, 128, 128),
        Color::LightGreen => (128, 255, 128),
        Color::LightYellow => (255, 255, 128),
        Color::LightBlue => (128, 128, 255),
        Color::LightMagenta => (255, 128, 255),
        Color::LightCyan => (128, 255, 255),
        Color::White => (255, 255, 255),
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255), // Default to white
    }
}
