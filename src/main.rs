#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use abf_reader::AbfReader;
use iced::widget::{button, checkbox, column, container, row, scrollable, slider, text};
use iced::{Element, Length};
use iced_plot::{Color, LineStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series};
use std::path::Path;

#[derive(Debug, Clone)]
enum Message {
    ToggleChannel(usize),
    ConfirmSelection,
    SetViewDuration(f64),
    ResetZoom,
    PanLeft,
    PanRight,
    PlotMessage(usize, PlotUiMessage),
}

enum AppState {
    Selecting {
        channel_selected: [bool; 9],
    },
    #[allow(dead_code)]
    Viewing {
        widgets: Vec<iced_plot::PlotWidget>,
        channels: Vec<Vec<f32>>,
        channel_selected: [bool; 9],
        sample_rate: f64,
        channel_names: Vec<String>,
        channel_units: Vec<String>,
        total_duration: f64,
        view_duration: f64,
        time_offset: f64,
    },
}

fn main() -> iced::Result {
    iced::application(app_new, app_update, app_view).window_size((1200.0, 1400.0)).run()
}

fn app_new() -> AppState {
    AppState::Selecting { channel_selected: [true; 9] }
}

fn app_update(state: &mut AppState, message: Message) {
    match state {
        AppState::Selecting { channel_selected } => match message {
            Message::ToggleChannel(idx) => {
                if idx < 9 {
                    channel_selected[idx] = !channel_selected[idx];
                }
            }
            Message::ConfirmSelection => {
                let (channels, sample_rate, channel_names, channel_units, total_duration) = read_channel_data(*channel_selected, 0.0, f64::INFINITY);
                // Default view to 5 seconds or full duration if shorter
                let default_view = 5.0_f64.min(total_duration);
                let widgets = build_zoomed_widgets_data(
                    &channels,
                    channel_selected,
                    sample_rate,
                    &channel_names,
                    &channel_units,
                    default_view,
                    0.0,
                );
                *state = AppState::Viewing {
                    widgets,
                    channels,
                    channel_selected: *channel_selected,
                    sample_rate,
                    channel_names,
                    channel_units,
                    total_duration,
                    view_duration: default_view,
                    time_offset: 0.0,
                };
            }
            _ => {}
        },
        AppState::Viewing {
            widgets,
            channels,
            channel_selected,
            sample_rate,
            channel_names,
            channel_units,
            total_duration,
            view_duration,
            time_offset,
        } => match message {
            Message::SetViewDuration(duration) => {
                *view_duration = duration.max(0.1).min(*total_duration);
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                );
            }
            Message::ResetZoom => {
                *view_duration = 5.0_f64.min(*total_duration);
                *time_offset = 0.0;
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                );
            }
            Message::PanLeft => {
                *time_offset = (*time_offset - *view_duration / 4.0).max(0.0);
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                );
            }
            Message::PanRight => {
                *time_offset = (*time_offset + *view_duration / 4.0).min((*total_duration - *view_duration).max(0.0));
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                );
            }
            Message::PlotMessage(_idx, plot_msg) => {
                // Apply to all widgets (linked plots)
                for widget in widgets.iter_mut() {
                    widget.update(plot_msg.clone());
                }
            }
            _ => {}
        },
    }
}

fn app_view(state: &AppState) -> Element<'_, Message> {
    match state {
        AppState::Selecting { channel_selected } => selection_view(channel_selected),
        AppState::Viewing {
            widgets,
            channels: _,
            channel_selected,
            sample_rate: _,
            channel_names: _,
            channel_units: _,
            total_duration,
            view_duration,
            time_offset,
        } => {
            let selected_count = channel_selected.iter().filter(|&&b| b).count();
            let base_height = if selected_count <= 2 {
                400.0
            } else if selected_count <= 4 {
                300.0
            } else if selected_count <= 6 {
                200.0
            } else {
                140.0
            };

            let zoom_controls = container(
                row![
                    text(format!("Pos: {:.1}s", time_offset)).size(11).width(Length::Fixed(80.0)),
                    text("View Duration:").size(11).width(Length::Fixed(90.0)),
                    slider(0.1..=*total_duration, *view_duration, Message::SetViewDuration).width(Length::Fixed(150.0)),
                    text(format!("{:.1}s", view_duration)).size(10).width(Length::Fixed(50.0)),
                    button("◄").on_press(Message::PanLeft).padding(5),
                    button("►").on_press(Message::PanRight).padding(5),
                    button("Reset").on_press(Message::ResetZoom).padding(5),
                ]
                .spacing(8),
            )
            .padding(10)
            .width(Length::Fill);

            let mut plots_col = column![].spacing(0);

            for (index, widget) in widgets.iter().enumerate() {
                let plot = widget.view().map(move |msg| Message::PlotMessage(index, msg));

                let ch_label = text(format!("Ch{}", index)).size(11);
                let row_layout = row![
                    container(ch_label).width(Length::Fixed(45.0)),
                    container(plot).height(Length::Fixed(base_height as f32)).width(Length::Fill)
                ]
                .spacing(0)
                .width(Length::Fill);

                plots_col = plots_col.push(row_layout);
            }

            let header = container(column![text(format!("Loaded {} channels", selected_count)).size(14), zoom_controls,].spacing(5)).padding(10);

            column![header, scrollable(plots_col).height(Length::Fill).width(Length::Fill)].into()
        }
    }
}

fn selection_view(channel_selected: &[bool; 9]) -> Element<'_, Message> {
    let mut channel_col = column![text("Select Channels:").size(16)].spacing(5).padding(10);

    for (idx, selected) in channel_selected.iter().enumerate() {
        let label = format!("Channel {} (ch{})", idx, idx);
        let ch_checkbox = checkbox(*selected).label(label).on_toggle(move |_| Message::ToggleChannel(idx));
        channel_col = channel_col.push(ch_checkbox);
    }

    let confirm_btn = button("Load All Data and View").on_press(Message::ConfirmSelection).padding(10);

    let main_col = column![
        container(text("ABF Analytics Viewer").size(24)).padding(20).width(Length::Fill),
        scrollable(channel_col).height(Length::Fill),
        confirm_btn
    ]
    .spacing(10)
    .padding(20);

    container(main_col).height(Length::Fill).width(Length::Fill).into()
}

fn build_zoomed_widgets_data(
    channels: &Vec<Vec<f32>>,
    channel_selected: &[bool; 9],
    sample_rate: f64,
    channel_names: &Vec<String>,
    channel_units: &Vec<String>,
    view_duration: f64,
    time_offset: f64,
) -> Vec<PlotWidget> {
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

    channels
        .iter()
        .enumerate()
        .filter(|(idx, _)| channel_selected[*idx])
        .flat_map(|(channel_idx, channel)| {
            // Keep original data points
            let positions: Vec<[f64; 2]> = channel
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    let time_sec = (i as f64) / sample_rate;
                    [time_sec, value as f64]
                })
                .collect();

            // Calculate axis bounds for zooming
            // Filter positions to show only the visible time region
            let zoomed_x_max = time_offset + view_duration;

            // Filter to visible region
            let mut filtered_positions: Vec<[f64; 2]> = positions
                .iter()
                .filter(|pos| {
                    let time = pos[0];
                    time >= time_offset && time <= zoomed_x_max
                })
                .copied()
                .collect();

            // Smart downsampling: if we have too many points, skip some
            // This keeps rendering fast while maintaining fidelity when zoomed in
            const MAX_POINTS: usize = 3000;
            if filtered_positions.len() > MAX_POINTS && filtered_positions.len() > 0 {
                let skip = (filtered_positions.len() / MAX_POINTS).max(1);
                let downsampled: Vec<[f64; 2]> = filtered_positions
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| i % skip == 0)
                    .map(|(_, p)| *p)
                    .collect();
                filtered_positions = downsampled;
            }

            // Calculate Y-axis bounds from filtered data BEFORE creating series
            let y_values: Vec<f64> = filtered_positions.iter().map(|p| p[1]).collect();
            let y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
            let y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let y_range = y_max - y_min;
            let padding = if y_range > 0.0 { y_range * 0.1 } else { 0.5 };

            let color = colors[channel_idx % colors.len()];

            let ch_name = channel_names
                .get(channel_idx)
                .map(|n| if n.is_empty() { format!("Channel {}", channel_idx) } else { n.clone() })
                .unwrap_or_else(|| format!("Channel {}", channel_idx));

            let series = Series::line_only(filtered_positions, LineStyle::Solid)
                .with_label(format!("{}", ch_name))
                .with_color(color);

            let unit = channel_units
                .get(channel_idx)
                .map(|u| if u.is_empty() { "Value".to_string() } else { u.clone() })
                .unwrap_or_else(|| "Value".to_string());

            PlotWidgetBuilder::new()
                .add_series(series)
                .with_cursor_overlay(true)
                .with_x_label("Time (s)")
                .with_y_lim(y_min - padding, y_max + padding)
                .with_y_label(&unit)
                .with_crosshairs(true)
                .disable_legend()
                .disable_controls_help()
                .build()
                .ok()
        })
        .collect()
}

fn read_channel_data(_channel_selected: [bool; 9], _start_offset: f64, _duration: f64) -> (Vec<Vec<f32>>, f64, Vec<String>, Vec<String>, f64) {
    let mut channels = Vec::new();
    let mut sample_rate = 1.0;
    let mut channel_units: Vec<String> = Vec::new();
    let mut channel_names: Vec<String> = Vec::new();
    let mut total_duration = 0.0;

    if let Ok(mut reader) = AbfReader::open_with_options(
        Path::new("Example.abf"),
        abf_reader::AbfHeaderReadOptions {
            group5_hardware: true,
            group7_multichannel: true,
            ext_group7_multichannel: true,
            ..Default::default()
        },
    ) {
        sample_rate = reader.get_sample_rate();
        total_duration = reader.get_duration_seconds();

        let num_channels = reader.header.group3_trial_hierarchy.as_ref().unwrap().adc_num_channels as usize;
        for ch_idx in 0..num_channels {
            channel_names.push(reader.get_adc_channel_name(ch_idx));
            channel_units.push(reader.get_adc_unit(ch_idx));
        }

        // Load entire file (ignore start_offset and duration params, always load all)
        if let Ok(data) = reader.read_channels_time_window(0.0, total_duration) {
            channels = data;
            println!("Loaded {} channels", channels.len());
            println!("Sample rate: {:.0} Hz", sample_rate);
            println!("Total duration: {:.2} seconds ({:.2} minutes)", total_duration, total_duration / 60.0);
            println!("Entire file loaded");
            for (idx, (name, unit)) in channel_names.iter().zip(channel_units.iter()).enumerate() {
                println!("  Channel {}: {} ({})", idx, name, unit);
                if !channels.is_empty() && !channels[idx].is_empty() {
                    let ch = &channels[idx];
                    let min_val = ch.iter().cloned().fold(f32::INFINITY, f32::min);
                    let max_val = ch.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let avg_val = ch.iter().sum::<f32>() / ch.len() as f32;
                    println!("    Value range: {:.4} to {:.4} (avg: {:.4})", min_val, max_val, avg_val);
                }
            }

            if let Some(group5) = &reader.header.group5_hardware {
                println!("\nHardware Scaling (Group 5):");
                println!("  ADC Range: {}", group5.adc_range);
                println!("  ADC Resolution: {}", group5.adc_resolution);
            }
            if let Some(group7) = &reader.header.group7_multichannel {
                println!("\nChannel Scaling (Group 7):");
                for i in 0..3.min(num_channels) {
                    println!(
                        "  Channel {}: InstrumentScaleFactor={}, ADCGain={}, Offset={}",
                        i, group7.instrument_scale_factor[i], group7.adc_programmable_gain[i], group7.instrument_offset[i]
                    );
                }
            }
        }
    }

    (channels, sample_rate, channel_names, channel_units, total_duration)
}
