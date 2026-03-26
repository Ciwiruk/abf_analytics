#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use abf_reader::AbfReader;
use iced::widget::{column, container, scrollable};
use iced::{Element, Length};
use iced_plot::{Color, LineStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series};
use std::path::Path;

#[derive(Debug, Clone)]
enum Message {
    PlotMessage(usize, PlotUiMessage),
}

fn main() -> iced::Result {
    iced::application(new, update, view).window_size((1200.0, 1400.0)).run()
}

fn update(widgets: &mut Vec<PlotWidget>, message: Message) {
    if let Message::PlotMessage(index, plot_msg) = message {
        if let Some(widget) = widgets.get_mut(index) {
            widget.update(plot_msg);
        }
    }
}

fn view(widgets: &Vec<PlotWidget>) -> Element<'_, Message> {
    let mut col = column![container(iced::widget::text(format!("Loaded {} channels", widgets.len()))).padding(10)]
        .spacing(20)
        .padding(10);

    for (index, widget) in widgets.iter().enumerate() {
        let title = container(iced::widget::text(format!("Channel {}", index)).size(14)).padding(5);

        let plot = widget.view().map(move |msg| Message::PlotMessage(index, msg));
        col = col.push(title).push(container(plot).height(Length::Fixed(350.0)).width(Length::Fill));
    }

    let scrollable_col = scrollable(col).height(Length::Fill).width(Length::Fill);
    container(scrollable_col).height(Length::Fill).width(Length::Fill).into()
}

fn new() -> Vec<PlotWidget> {
    let mut channels = Vec::new();
    if let Ok(mut reader) = AbfReader::open(Path::new("Example.abf")) {
        if let Ok(data) = reader.read_channels() {
            channels = data;
            println!("Loaded {} channels", channels.len());
        }
    }

    let colors = [
        Color::from_rgb(0.3, 0.3, 0.9),
        Color::from_rgb(0.9, 0.3, 0.3),
        Color::from_rgb(0.3, 0.9, 0.3),
        Color::from_rgb(0.9, 0.9, 0.3),
        Color::from_rgb(0.9, 0.3, 0.9),
        Color::from_rgb(0.3, 0.9, 0.9),
        Color::from_rgb(0.8, 0.5, 0.2),
        Color::from_rgb(0.5, 0.2, 0.8),
        Color::from_rgb(0.2, 0.8, 0.5),
    ];

    // Create a separate graph for each channel
    channels
        .iter()
        .enumerate()
        .map(|(channel_idx, channel)| {
            let positions: Vec<[f64; 2]> = channel.iter().take(10).enumerate().map(|(i, &value)| [i as f64, value as f64]).collect();

            let color = colors[channel_idx % colors.len()];
            let series = Series::line_only(positions, LineStyle::Solid)
                .with_label(format!("Channel {}", channel_idx))
                .with_color(color);

            PlotWidgetBuilder::new()
                .add_series(series)
                .with_cursor_overlay(true)
                .with_cursor_provider(|x, y| format!("X: {x:.2}, Y: {y:.2}"))
                .with_x_label("Sample Index")
                .with_y_label("Value")
                .with_crosshairs(true)
                .build()
                .unwrap()
        })
        .collect()
}
