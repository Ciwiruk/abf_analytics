#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use abf_reader::AbfReader;
use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Element, Length};
use iced_plot::{AxisLink, Color, LineStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalysisType {
    RawSignal,
    PeakDetection,
}

impl AnalysisType {
    fn all() -> Vec<Self> {
        vec![AnalysisType::RawSignal, AnalysisType::PeakDetection]
    }
}

impl std::fmt::Display for AnalysisType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnalysisType::RawSignal => write!(f, "Raw Signal"),
            AnalysisType::PeakDetection => write!(f, "Peak Detection"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ToggleChannel(usize),
    ConfirmSelection,
    ResetZoom,
    PanLeft,
    PanRight,
    SetPositionInput(String),
    SetViewWindowInput(String),
    UpdateViewSettings,
    PlotMessage(usize, PlotUiMessage),
    DetectPeaks,
    SwitchToViewing,
    SwitchToAnalytics,
    SelectAnalysisType(AnalysisType),
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
        position_input: String,
        view_window_input: String,
        x_axis_link: AxisLink,
    },
    #[allow(dead_code)]
    Analytics {
        widget: PlotWidget,
        channels: Vec<Vec<f32>>,
        channel_selected: [bool; 9],
        sample_rate: f64,
        channel_names: Vec<String>,
        channel_units: Vec<String>,
        selected_channel: usize,
        analysis_type: AnalysisType,
        analysis_peaks: Vec<[f64; 2]>,
        x_axis_link: AxisLink,
        total_duration: f64,
        view_duration: f64,
        time_offset: f64,
        position_input: String,
        view_window_input: String,
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
                let x_axis_link = AxisLink::new();
                let widgets = build_zoomed_widgets_data(
                    &channels,
                    channel_selected,
                    sample_rate,
                    &channel_names,
                    &channel_units,
                    default_view,
                    0.0,
                    &x_axis_link,
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
                    position_input: "0.0".to_string(),
                    view_window_input: format!("{:.1}", default_view),
                    x_axis_link,
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
            position_input,
            view_window_input,
            x_axis_link,
        } => match message {
            Message::ResetZoom => {
                *position_input = "0.0".to_string();
                *view_window_input = format!("{:.1}", view_duration);
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                    x_axis_link,
                );
            }
            Message::PanLeft => {
                *time_offset = (*time_offset - *view_duration / 4.0).max(0.0);
                *position_input = format!("{:.1}", time_offset);
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                    x_axis_link,
                );
            }
            Message::PanRight => {
                *time_offset = (*time_offset + *view_duration / 4.0).min((*total_duration - *view_duration).max(0.0));
                *position_input = format!("{:.1}", time_offset);
                *widgets = build_zoomed_widgets_data(
                    channels,
                    channel_selected,
                    *sample_rate,
                    channel_names,
                    channel_units,
                    *view_duration,
                    *time_offset,
                    x_axis_link,
                );
            }
            Message::SetPositionInput(input) => {
                *position_input = input;
            }
            Message::SetViewWindowInput(input) => {
                *view_window_input = input;
            }
            Message::UpdateViewSettings => {
                let pos_result = position_input.parse::<f64>();
                let dur_result = view_window_input.parse::<f64>();

                if let (Ok(new_offset), Ok(new_duration)) = (pos_result, dur_result) {
                    *view_duration = new_duration.max(0.1).min(*total_duration);
                    *time_offset = new_offset.max(0.0).min((*total_duration - *view_duration).max(0.0));
                    *position_input = format!("{:.1}", time_offset);
                    *view_window_input = format!("{:.1}", view_duration);
                    *widgets = build_zoomed_widgets_data(
                        channels,
                        channel_selected,
                        *sample_rate,
                        channel_names,
                        channel_units,
                        *view_duration,
                        *time_offset,
                        x_axis_link,
                    );
                }
            }
            Message::PlotMessage(idx, plot_msg) => {
                // Only update the specific plot that was interacted with
                if let Some(widget) = widgets.get_mut(idx) {
                    widget.update(plot_msg);
                }
            }
            Message::SwitchToAnalytics => {
                if let Some(ch_idx) = channel_selected.iter().position(|&b| b) {
                    let start_idx = (*time_offset * *sample_rate).floor() as usize;
                    let end_idx = ((*time_offset + *view_duration) * *sample_rate).ceil() as usize;
                    let mut peaks = detect_peaks_simple(&channels[ch_idx][start_idx..end_idx], *sample_rate);
                    for p in peaks.iter_mut() {
                        p[0] += *time_offset;
                    }
                    let default_view = 5.0_f64.min(*total_duration);
                    let analysis_type = AnalysisType::PeakDetection;
                    let widget = build_analysis_plot(
                        &channels[ch_idx],
                        &channel_names[ch_idx],
                        &channel_units[ch_idx],
                        &peaks,
                        *sample_rate,
                        default_view,
                        0.0,
                        x_axis_link,
                        analysis_type,
                    );
                    *state = AppState::Analytics {
                        widget,
                        channels: channels.clone(),
                        channel_selected: *channel_selected,
                        sample_rate: *sample_rate,
                        channel_names: channel_names.clone(),
                        channel_units: channel_units.clone(),
                        selected_channel: ch_idx,
                        analysis_type,
                        analysis_peaks: peaks,
                        x_axis_link: x_axis_link.clone(),
                        total_duration: *total_duration,
                        view_duration: default_view,
                        time_offset: 0.0,
                        position_input: "0.0".to_string(),
                        view_window_input: format!("{:.1}", default_view),
                    };
                }
            }
            _ => {}
        },
        AppState::Analytics {
            widget,
            channels,
            channel_selected: _,
            sample_rate,
            channel_names,
            channel_units,
            selected_channel,
            analysis_type,
            analysis_peaks,
            x_axis_link,
            total_duration,
            view_duration,
            time_offset,
            position_input,
            view_window_input,
        } => match message {
            Message::DetectPeaks => {
                let start_idx = (*time_offset * *sample_rate).floor() as usize;
                let end_idx = ((*time_offset + *view_duration) * *sample_rate).ceil() as usize;
                let mut peaks = detect_peaks_simple(&channels[*selected_channel][start_idx..end_idx], *sample_rate);
                for p in peaks.iter_mut() {
                    p[0] += *time_offset;
                }
                *analysis_peaks = peaks.clone();
                *widget = build_analysis_plot(
                    &channels[*selected_channel],
                    &channel_names[*selected_channel],
                    &channel_units[*selected_channel],
                    &peaks,
                    *sample_rate,
                    *view_duration,
                    *time_offset,
                    x_axis_link,
                    *analysis_type,
                );
            }
            Message::SelectAnalysisType(atype) => {
                *analysis_type = atype;
                *widget = build_analysis_plot(
                    &channels[*selected_channel],
                    &channel_names[*selected_channel],
                    &channel_units[*selected_channel],
                    &analysis_peaks,
                    *sample_rate,
                    *view_duration,
                    *time_offset,
                    x_axis_link,
                    atype,
                );
            }
            Message::SetPositionInput(input) => {
                *position_input = input;
            }
            Message::SetViewWindowInput(input) => {
                *view_window_input = input;
            }
            Message::UpdateViewSettings => {
                let pos_result = position_input.parse::<f64>();
                let dur_result = view_window_input.parse::<f64>();

                if let (Ok(new_offset), Ok(new_duration)) = (pos_result, dur_result) {
                    *view_duration = new_duration.max(0.1).min(*total_duration);
                    *time_offset = new_offset.max(0.0).min((*total_duration - *view_duration).max(0.0));
                    *position_input = format!("{:.1}", time_offset);
                    *view_window_input = format!("{:.1}", view_duration);
                    *widget = build_analysis_plot(
                        &channels[*selected_channel],
                        &channel_names[*selected_channel],
                        &channel_units[*selected_channel],
                        &analysis_peaks,
                        *sample_rate,
                        *view_duration,
                        *time_offset,
                        x_axis_link,
                        *analysis_type,
                    );
                }
            }
            Message::PlotMessage(_, _) => {}
            Message::SwitchToViewing => {}
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
            total_duration: _,
            view_duration: _,
            time_offset: _,
            position_input,
            view_window_input,
            x_axis_link: _,
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

            let position_row = row![
                text("Position (s):").size(11).width(Length::Fixed(80.0)),
                text_input("0.0", position_input)
                    .on_input(Message::SetPositionInput)
                    .width(Length::Fixed(80.0))
                    .padding(5),
            ]
            .spacing(5);

            let view_window_row = row![
                text("View Window (s):").size(11).width(Length::Fixed(100.0)),
                text_input("5.0", view_window_input)
                    .on_input(Message::SetViewWindowInput)
                    .width(Length::Fixed(80.0))
                    .padding(5),
            ]
            .spacing(5);

            let control_buttons = row![
                button("View Signals").on_press(Message::SwitchToViewing).padding(5),
                button("◄ Pan").on_press(Message::PanLeft).padding(5),
                button("Pan ►").on_press(Message::PanRight).padding(5),
                button("Update").on_press(Message::UpdateViewSettings).padding(5),
                button("Reset").on_press(Message::ResetZoom).padding(5),
                button("Analysis").on_press(Message::SwitchToAnalytics).padding(5),
            ]
            .spacing(8);

            let zoom_controls = container(column![position_row, view_window_row, control_buttons,].spacing(8))
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
        AppState::Analytics {
            widget,
            channels: _,
            channel_selected: _,
            sample_rate: _,
            channel_names: _,
            channel_units: _,
            selected_channel,
            analysis_type,
            analysis_peaks: _,
            x_axis_link: _,
            total_duration: _,
            view_duration: _,
            time_offset: _,
            position_input,
            view_window_input,
        } => {
            // Match Viewing tab layout exactly
            let position_row = row![
                text("Position (s):").size(11).width(Length::Fixed(80.0)),
                text_input("0.0", position_input)
                    .on_input(Message::SetPositionInput)
                    .width(Length::Fixed(80.0))
                    .padding(5),
            ]
            .spacing(5);

            let view_window_row = row![
                text("View Window (s):").size(11).width(Length::Fixed(100.0)),
                text_input("5.0", view_window_input)
                    .on_input(Message::SetViewWindowInput)
                    .width(Length::Fixed(80.0))
                    .padding(5),
            ]
            .spacing(5);

            let analysis_type_row = row![
                text("Analysis Type:").size(11).width(Length::Fixed(90.0)),
                pick_list(AnalysisType::all(), Some(*analysis_type), Message::SelectAnalysisType).width(Length::Fixed(120.0)),
            ]
            .spacing(5);

            let control_buttons = row![
                button("View Signals").on_press(Message::SwitchToViewing).padding(5),
                button("◄ Pan").on_press(Message::PanLeft).padding(5),
                button("Pan ►").on_press(Message::PanRight).padding(5),
                button("Update").on_press(Message::UpdateViewSettings).padding(5),
                button("Reset").on_press(Message::ResetZoom).padding(5),
                button("Detect Peaks").on_press(Message::DetectPeaks).padding(5),
            ]
            .spacing(8);

            let zoom_controls = container(column![position_row, view_window_row, analysis_type_row, control_buttons,].spacing(8))
                .padding(10)
                .width(Length::Fill);

            // Display single channel in same format as Viewing tab
            let plot = widget.view().map(move |_msg| Message::PlotMessage(0, _msg));

            let ch_label = text(format!("Ch{}", selected_channel)).size(11);
            let row_layout = row![
                container(ch_label).width(Length::Fixed(45.0)),
                container(plot).height(Length::Fixed(400.0)).width(Length::Fill)
            ]
            .spacing(0)
            .width(Length::Fill);

            let plots_col = column![row_layout].spacing(0);

            let header = container(column![text("Loaded 1 channel (Analysis)").size(14), zoom_controls,].spacing(5)).padding(10);

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
    x_axis_link: &AxisLink,
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

            // Min-max downsampling for oscillating signals (EKG-like) - OPTIMIZED single pass
            // Divides view into buckets and keeps min/max from each bucket
            // This preserves peaks and valleys while reducing point count
            const MAX_POINTS: usize = 2400;
            if filtered_positions.len() > MAX_POINTS && filtered_positions.len() > 1 {
                let num_buckets = (MAX_POINTS / 2).max(10);
                let bucket_width = view_duration / (num_buckets as f64);

                // Store min/max per bucket in a single pass through data
                let mut buckets: Vec<Option<([f64; 2], [f64; 2])>> = vec![None; num_buckets];

                for point in &filtered_positions {
                    let relative_time = point[0] - time_offset;
                    if relative_time >= 0.0 && relative_time <= view_duration {
                        let bucket_idx = ((relative_time / bucket_width).floor() as usize).min(num_buckets - 1);

                        match &mut buckets[bucket_idx] {
                            None => {
                                buckets[bucket_idx] = Some((*point, *point));
                            }
                            Some((min_pt, max_pt)) => {
                                if point[1] < min_pt[1] {
                                    *min_pt = *point;
                                }
                                if point[1] > max_pt[1] {
                                    *max_pt = *point;
                                }
                            }
                        }
                    }
                }

                // Build downsampled list from buckets
                let mut downsampled = Vec::new();
                for bucket_opt in buckets {
                    if let Some((min_pt, max_pt)) = bucket_opt {
                        // Add points in time order to maintain waveform shape
                        if min_pt[0] < max_pt[0] {
                            downsampled.push(min_pt);
                            downsampled.push(max_pt);
                        } else {
                            downsampled.push(max_pt);
                            downsampled.push(min_pt);
                        }
                    }
                }

                filtered_positions = downsampled;
            }

            // Calculate Y-axis bounds from FILTERED (visible) data for proper calibration display
            // Each channel shows bounds for what's actually visible in the current view
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
                .with_x_axis_link(x_axis_link.clone())
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

pub fn moving_average(data: &[f32], window_size: usize) -> Vec<f32> {
    if window_size == 0 {
        return data.to_vec();
    }
    let mut result = Vec::with_capacity(data.len());
    let mut sum = 0.0;
    for i in 0..data.len() {
        sum += data[i];
        if i >= window_size {
            sum -= data[i - window_size];
            result.push(sum / window_size as f32);
        } else {
            result.push(sum / (i + 1) as f32);
        }
    }
    result
}

pub fn moving_std(data: &[f32], window_size: usize) -> Vec<f32> {
    let n = data.len();
    if n == 0 || window_size == 0 {
        return vec![0.0; n];
    }

    let mut prefix_sum = vec![0.0_f64; n + 1];
    let mut prefix_sum_sq = vec![0.0_f64; n + 1];
    for i in 0..n {
        prefix_sum[i + 1] = prefix_sum[i] + data[i] as f64;
        prefix_sum_sq[i + 1] = prefix_sum_sq[i] + (data[i] as f64) * (data[i] as f64);
    }

    let mut out = vec![0.0_f32; n];
    for i in 0..n {
        let start = if i + 1 >= window_size { i + 1 - window_size } else { 0 };
        let end = i + 1;
        let len = (end - start) as f64;
        let sum = prefix_sum[end] - prefix_sum[start];
        let sumsq = prefix_sum_sq[end] - prefix_sum_sq[start];
        let mean = sum / len;
        let var = (sumsq / len) - (mean * mean);
        out[i] = if var > 0.0 { var.sqrt() as f32 } else { 0.0 };
    }
    out
}

pub fn detect_peaks_simple(data: &[f32], sample_rate: f64) -> Vec<[f64; 2]> {
    let window_size = 2usize; // neighborhood for argmax around candidate
    let data_smoothed = moving_average(data, window_size);

    if data_smoothed.len() < 3 {
        return Vec::new();
    }

    // Local adaptive window (seconds) -> samples
    let local_window_sec = 20.0_f64; // tune 0.2..1.0 for your data
    let mut local_window = ((sample_rate * local_window_sec).round() as usize).max(1);
    if local_window > data_smoothed.len() {
        local_window = data_smoothed.len();
    }

    // Local mean/std computed on smoothed signal
    let local_mean = moving_average(&data_smoothed, local_window);
    let local_std = moving_std(&data_smoothed, local_window);

    let k = 1.6666667_f32; // sensitivity multiplier: lower => more detections

    let min_distance_sec = 0.02; // minimum separation in seconds (tune or compute per-record)

    let mut peaks: Vec<[f64; 2]> = Vec::new();
    let mut last_peak_time = -min_distance_sec;
    let mut last_peak_value = f32::MIN;

    for i in 1..data_smoothed.len() - 1 {
        let val_smoothed = data_smoothed[i];
        let val = data[i];

        let thr_i = local_mean[i] + k * local_std[i];

        if (val_smoothed > thr_i && val_smoothed > data_smoothed[i - 1] && val_smoothed > data_smoothed[i + 1])
            || (val > thr_i && val > data[i - 1] && val > data[i + 1])
        {
            let neigh_start = i.saturating_sub(window_size);
            let neigh_end = (i + window_size + 1).min(data.len()); // exclusive

            // argmax in neighborhood on raw data
            let (rel_idx, max_val) = data[neigh_start..neigh_end].iter().copied().enumerate().fold(
                (0usize, f32::NEG_INFINITY),
                |(ai, am), (j, x)| {
                    if x > am {
                        (j, x)
                    } else {
                        (ai, am)
                    }
                },
            );

            let peak_index = neigh_start + rel_idx;
            let peak_time = (peak_index as f64) / sample_rate;

            if peak_time - last_peak_time >= min_distance_sec {
                if i != 1 {
                    peaks.push([peak_time, max_val as f64]);
                }

                last_peak_value = max_val;
                last_peak_time = peak_time;
            } else {
                // too close: keep the higher peak (replace if current is higher)
                if max_val > last_peak_value {
                    if let Some(last) = peaks.last_mut() {
                        *last = [peak_time, max_val as f64];
                    }
                    last_peak_value = max_val;
                    last_peak_time = peak_time;
                }
            }
        }
    }

    // optional debug
    println!("Peaks detected: {}", peaks.len());

    peaks
}

fn build_analysis_plot(
    channel_data: &[f32],
    channel_name: &str,
    channel_unit: &str,
    peaks: &[[f64; 2]],
    sample_rate: f64,
    view_duration: f64,
    time_offset: f64,
    x_axis_link: &AxisLink,
    analysis_type: AnalysisType,
) -> PlotWidget {
    // Build signal data
    let signal_points: Vec<[f64; 2]> = channel_data
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            let time_sec = (i as f64) / sample_rate;
            [time_sec, value as f64]
        })
        .collect();

    if signal_points.is_empty() {
        return PlotWidgetBuilder::new()
            .with_cursor_overlay(true)
            .with_x_label("Time (s)")
            .with_y_label(channel_unit)
            .build()
            .expect("Failed to build analysis plot");
    }

    // Filter to visible region
    let zoomed_x_max = time_offset + view_duration;
    let mut filtered_positions: Vec<[f64; 2]> = signal_points
        .iter()
        .filter(|pos| {
            let time = pos[0];
            time >= time_offset && time <= zoomed_x_max
        })
        .copied()
        .collect();

    // Downsample if too many points (prevent buffer size error)
    const MAX_POINTS: usize = 5000;
    if filtered_positions.len() > MAX_POINTS {
        let num_buckets = (MAX_POINTS / 2).max(10);
        let bucket_width = view_duration / (num_buckets as f64);
        let mut buckets: Vec<Option<([f64; 2], [f64; 2])>> = vec![None; num_buckets];

        for point in &filtered_positions {
            let relative_time = point[0] - time_offset;
            if relative_time >= 0.0 && relative_time <= view_duration {
                let bucket_idx = ((relative_time / bucket_width).floor() as usize).min(num_buckets - 1);

                match &mut buckets[bucket_idx] {
                    None => {
                        buckets[bucket_idx] = Some((*point, *point));
                    }
                    Some((min_pt, max_pt)) => {
                        if point[1] < min_pt[1] {
                            *min_pt = *point;
                        }
                        if point[1] > max_pt[1] {
                            *max_pt = *point;
                        }
                    }
                }
            }
        }

        let mut downsampled = Vec::new();
        for bucket_opt in buckets {
            if let Some((min_pt, max_pt)) = bucket_opt {
                if min_pt[0] < max_pt[0] {
                    downsampled.push(min_pt);
                    downsampled.push(max_pt);
                } else {
                    downsampled.push(max_pt);
                    downsampled.push(min_pt);
                }
            }
        }

        filtered_positions = downsampled;
    }

    let signal_series = Series::line_only(filtered_positions.clone(), LineStyle::Solid)
        .with_label(format!("{}", channel_name))
        .with_color(Color::from_rgb(0.3, 0.3, 0.9));

    // Build plot with signal
    let mut builder = PlotWidgetBuilder::new().add_series(signal_series);

    // Filter and add peaks only if PeakDetection analysis type
    if matches!(analysis_type, AnalysisType::PeakDetection) && !peaks.is_empty() {
        let filtered_peaks: Vec<[f64; 2]> = peaks.iter().filter(|p| p[0] >= time_offset && p[0] <= zoomed_x_max).copied().collect();

        if !filtered_peaks.is_empty() {
            // Create vertical lines for each peak (better visualization)
            // Draw from y_min to peak value as small vertical markers
            let mut peak_markers: Vec<[f64; 2]> = Vec::new();
            for peak in &filtered_peaks {
                // Add peak point itself
                peak_markers.push(*peak);
            }

            let peaks_series = Series::line_only(peak_markers, LineStyle::Solid)
                .with_label("Peaks")
                .with_color(Color::from_rgb(1.0, 0.2, 0.2));
            builder = builder.add_series(peaks_series);
        }
    }

    // Calculate Y-axis bounds from filtered data
    let y_values: Vec<f64> = filtered_positions.iter().map(|p| p[1]).collect();
    let y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_range = y_max - y_min;
    let padding = if y_range > 0.0 { y_range * 0.1 } else { 0.5 };

    builder
        .with_cursor_overlay(true)
        .with_x_label("Time (s)")
        .with_x_axis_link(x_axis_link.clone())
        .with_y_lim(y_min - padding, y_max + padding)
        .with_y_label(channel_unit)
        .with_crosshairs(true)
        .disable_legend()
        .disable_controls_help()
        .build()
        .expect("Failed to build analysis plot")
}
