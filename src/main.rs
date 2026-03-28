#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use abf_reader::AbfReader;
use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, slider, space, text, text_input};
use iced::{Center, Element, Fill};
use iced_plot::{AxisLink, Color, LineStyle, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalysisType {
    RawSignal,
    PeakDetection,
    FourierTransform,
}

impl AnalysisType {
    fn all() -> Vec<Self> {
        vec![AnalysisType::RawSignal, AnalysisType::PeakDetection, AnalysisType::FourierTransform]
    }
}

impl std::fmt::Display for AnalysisType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnalysisType::RawSignal => write!(f, "Raw Signal"),
            AnalysisType::PeakDetection => write!(f, "Peak Detection"),
            AnalysisType::FourierTransform => write!(f, "Fourier Transform"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ToggleChannel(usize),
    ToggleChannelTemp(usize),
    ConfirmSelection,
    ConfirmChannelSelection,
    CancelChannelSelection,
    ToggleChannelSelector,
    SetGraphHeight(f32),
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
    ToggleCustomSineWaves,
    SetCustomFrequencyInput(String),
    SetCustomAmplitudeInput(String),
    SetCustomPhaseInput(String),
    AddCustomSineWave,
    RemoveCustomSineWave(usize),
    ToggleFFTOnPeaks,
    ToggleFFTSmoothing,
    OpenFileDialog,
    SetReconstructionPositionInput(String),
    SetReconstructionViewWindowInput(String),
    UpdateReconstructionViewSettings,
    SetReconstructionOffsetSlider(String),
}

enum Screen {
    Selecting,
    Viewing { widgets: Vec<iced_plot::PlotWidget> },
    Analytics { widgets: Vec<PlotWidget>, analysis_type: AnalysisType },
}

struct AbfAnalytics {
    // Shared state across all screens
    channels: Vec<Vec<f32>>,
    channel_selected: [bool; 9],
    channel_selected_temp: [bool; 9],
    sample_rate: f64,
    channel_names: Vec<String>,
    channel_units: Vec<String>,
    total_duration: f64,
    position_input: String,
    view_window_input: String,

    // View/Analytics shared state
    time_offset: f64,
    view_duration: f64,
    x_axis_link: AxisLink,
    show_channel_selector: bool,
    graph_height: f32,

    // Custom sine waves for FFT overlay
    show_custom_sine_waves: bool,
    custom_frequency_input: String,
    custom_amplitude_input: String,
    custom_phase_input: String,
    custom_sine_waves: Vec<(f64, f64, f64)>, // (frequency Hz, amplitude, phase radians)
    reconstruction_widget: Option<PlotWidget>,
    reconstruction_position_input: String,    // time offset input for reconstruction
    reconstruction_view_window_input: String, // window duration input for reconstruction
    reconstruction_time_offset: f64,          // separate time offset for reconstruction plot
    reconstruction_view_duration: f64,        // separate view duration for reconstruction plot
    reconstruction_offset_input: String,      // offset input for sine reconstruction
    reconstruction_offset: f32,               // vertical offset for sine wave
    fft_on_peaks: bool,                       // perform FFT on detected peaks instead of raw signal
    fft_smoothing: bool,                      // smooth FFT magnitudes to reduce spectral noise

    // File selection on start screen
    selected_file_path: String,

    // Current screen state
    screen: Screen,
}

fn main() -> iced::Result {
    iced::application(AbfAnalytics::new, AbfAnalytics::update, AbfAnalytics::view)
        .window_size((1400.0, 900.0))
        .run()
}

impl AbfAnalytics {
    fn new() -> Self {
        AbfAnalytics {
            channels: Vec::new(),
            channel_selected: [true; 9],
            channel_selected_temp: [true; 9],
            sample_rate: 1.0,
            channel_names: Vec::new(),
            channel_units: Vec::new(),
            total_duration: 0.0,
            position_input: "0.0".to_string(),
            view_window_input: "5.0".to_string(),
            time_offset: 0.0,
            view_duration: 5.0,
            x_axis_link: AxisLink::new(),
            show_channel_selector: false,
            graph_height: 140.0,
            show_custom_sine_waves: false,
            custom_frequency_input: "10.0".to_string(),
            custom_amplitude_input: "1.0".to_string(),
            custom_phase_input: "0.0".to_string(),
            custom_sine_waves: Vec::new(),
            reconstruction_widget: None,
            reconstruction_position_input: "0.0".to_string(),
            reconstruction_view_window_input: "5.0".to_string(),
            reconstruction_time_offset: 0.0,
            reconstruction_view_duration: 5.0,
            reconstruction_offset_input: "0.0".to_string(),
            reconstruction_offset: 0.0,
            fft_on_peaks: false,
            fft_smoothing: false,
            selected_file_path: "Example.abf".to_string(),
            screen: Screen::Selecting,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleChannel(idx) => {
                if idx < 9 {
                    self.channel_selected[idx] = !self.channel_selected[idx];

                    // Rebuild widgets if in Viewing
                    if let Screen::Viewing { widgets } = &mut self.screen {
                        *widgets = build_zoomed_widgets_data(
                            &self.channels,
                            &self.channel_selected,
                            self.sample_rate,
                            &self.channel_names,
                            &self.channel_units,
                            self.view_duration,
                            self.time_offset,
                            &self.x_axis_link,
                        );
                    }
                    // Update widgets if in Analytics
                    else if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                        // Rebuild widgets for all selected channels
                        let mut new_widgets = Vec::new();
                        for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                            if selected {
                                let widget = build_analysis_plot(
                                    &self.channels[ch_idx],
                                    &self.channel_names[ch_idx],
                                    &self.channel_units[ch_idx],
                                    self.sample_rate,
                                    self.view_duration,
                                    self.time_offset,
                                    &self.x_axis_link,
                                    *analysis_type,
                                    &self.custom_sine_waves,
                                    self.fft_on_peaks,
                                    self.fft_smoothing,
                                );
                                new_widgets.push(widget);
                            }
                        }
                        *widgets = new_widgets;
                    }
                }
            }
            Message::ConfirmSelection => {
                let (channels, sample_rate, channel_names, channel_units, total_duration) =
                    read_channel_data(&self.selected_file_path, self.channel_selected, 0.0, f64::INFINITY);

                self.channels = channels;
                self.sample_rate = sample_rate;
                self.channel_names = channel_names.clone();
                self.channel_units = channel_units.clone();
                self.total_duration = total_duration;
                self.position_input = "0.0".to_string();

                let default_view = 5.0_f64.min(total_duration);
                self.view_window_input = format!("{:.1}", default_view);
                self.view_duration = default_view;
                self.time_offset = 0.0;
                self.x_axis_link = AxisLink::new();
                self.show_channel_selector = false;

                let widgets = build_zoomed_widgets_data(
                    &self.channels,
                    &self.channel_selected,
                    self.sample_rate,
                    &self.channel_names,
                    &self.channel_units,
                    default_view,
                    0.0,
                    &self.x_axis_link,
                );

                self.screen = Screen::Viewing { widgets };
            }
            Message::ToggleChannelSelector => {
                self.show_channel_selector = !self.show_channel_selector;
                if self.show_channel_selector {
                    // Copy current selection to temporary when opening popup
                    self.channel_selected_temp = self.channel_selected;
                }
            }
            Message::ToggleChannelTemp(idx) => {
                if idx < 9 {
                    self.channel_selected_temp[idx] = !self.channel_selected_temp[idx];
                }
            }
            Message::ConfirmChannelSelection => {
                self.channel_selected = self.channel_selected_temp;
                self.show_channel_selector = false;

                // Rebuild widgets if in Viewing
                if let Screen::Viewing { widgets } = &mut self.screen {
                    *widgets = build_zoomed_widgets_data(
                        &self.channels,
                        &self.channel_selected,
                        self.sample_rate,
                        &self.channel_names,
                        &self.channel_units,
                        self.view_duration,
                        self.time_offset,
                        &self.x_axis_link,
                    );
                }
                // Rebuild widgets if in Analytics
                else if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                    let mut new_widgets = Vec::new();
                    for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                        if selected {
                            let widget = build_analysis_plot(
                                &self.channels[ch_idx],
                                &self.channel_names[ch_idx],
                                &self.channel_units[ch_idx],
                                self.sample_rate,
                                self.view_duration,
                                self.time_offset,
                                &self.x_axis_link,
                                *analysis_type,
                                &self.custom_sine_waves,
                                self.fft_on_peaks,
                                self.fft_smoothing,
                            );
                            new_widgets.push(widget);
                        }
                    }
                    *widgets = new_widgets;
                }
            }
            Message::CancelChannelSelection => {
                self.show_channel_selector = false;
            }
            Message::SetGraphHeight(height) => {
                self.graph_height = height;
            }
            Message::ResetZoom => {
                if let Screen::Viewing { widgets } = &mut self.screen {
                    self.position_input = "0.0".to_string();
                    self.view_window_input = format!("{:.1}", self.view_duration);
                    self.time_offset = 0.0;
                    *widgets = build_zoomed_widgets_data(
                        &self.channels,
                        &self.channel_selected,
                        self.sample_rate,
                        &self.channel_names,
                        &self.channel_units,
                        self.view_duration,
                        self.time_offset,
                        &self.x_axis_link,
                    );
                }
            }
            Message::PanLeft => {
                if let Screen::Viewing { widgets } = &mut self.screen {
                    self.time_offset = (self.time_offset - self.view_duration / 4.0).max(0.0);
                    self.position_input = format!("{:.1}", self.time_offset);
                    *widgets = build_zoomed_widgets_data(
                        &self.channels,
                        &self.channel_selected,
                        self.sample_rate,
                        &self.channel_names,
                        &self.channel_units,
                        self.view_duration,
                        self.time_offset,
                        &self.x_axis_link,
                    );
                }
            }
            Message::PanRight => {
                if let Screen::Viewing { widgets } = &mut self.screen {
                    self.time_offset = (self.time_offset + self.view_duration / 4.0).min((self.total_duration - self.view_duration).max(0.0));
                    self.position_input = format!("{:.1}", self.time_offset);
                    *widgets = build_zoomed_widgets_data(
                        &self.channels,
                        &self.channel_selected,
                        self.sample_rate,
                        &self.channel_names,
                        &self.channel_units,
                        self.view_duration,
                        self.time_offset,
                        &self.x_axis_link,
                    );
                }
            }
            Message::SetPositionInput(input) => {
                self.position_input = input;
            }
            Message::SetViewWindowInput(input) => {
                self.view_window_input = input;
            }
            Message::UpdateViewSettings => {
                if let (Ok(new_offset), Ok(new_duration)) = (self.position_input.parse::<f64>(), self.view_window_input.parse::<f64>()) {
                    self.view_duration = new_duration.max(0.1).min(self.total_duration);
                    self.time_offset = new_offset.max(0.0).min((self.total_duration - self.view_duration).max(0.0));
                    self.position_input = format!("{:.1}", self.time_offset);
                    self.view_window_input = format!("{:.1}", self.view_duration);

                    match &mut self.screen {
                        Screen::Viewing { widgets } => {
                            *widgets = build_zoomed_widgets_data(
                                &self.channels,
                                &self.channel_selected,
                                self.sample_rate,
                                &self.channel_names,
                                &self.channel_units,
                                self.view_duration,
                                self.time_offset,
                                &self.x_axis_link,
                            );
                        }
                        Screen::Analytics { widgets, analysis_type } => {
                            let mut new_widgets = Vec::new();
                            for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                                if selected {
                                    let widget = build_analysis_plot(
                                        &self.channels[ch_idx],
                                        &self.channel_names[ch_idx],
                                        &self.channel_units[ch_idx],
                                        self.sample_rate,
                                        self.view_duration,
                                        self.time_offset,
                                        &self.x_axis_link,
                                        *analysis_type,
                                        &self.custom_sine_waves,
                                        self.fft_on_peaks,
                                        self.fft_smoothing,
                                    );
                                    new_widgets.push(widget);
                                }
                            }
                            *widgets = new_widgets;
                        }
                        _ => {}
                    }
                }
            }
            Message::PlotMessage(idx, plot_msg) => {
                if let Screen::Viewing { widgets } = &mut self.screen {
                    if let Some(widget) = widgets.get_mut(idx) {
                        widget.update(plot_msg);
                    }
                } else if let Screen::Analytics { widgets, .. } = &mut self.screen {
                    if let Some(widget) = widgets.get_mut(idx) {
                        widget.update(plot_msg);
                    }
                }
            }
            Message::SwitchToAnalytics => {
                let default_view = 5.0_f64.min(self.total_duration);
                let analysis_type = AnalysisType::PeakDetection;
                self.x_axis_link = AxisLink::new();

                // Build analysis plots for all selected channels
                let mut widgets = Vec::new();
                for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                    if selected {
                        let widget = build_analysis_plot(
                            &self.channels[ch_idx],
                            &self.channel_names[ch_idx],
                            &self.channel_units[ch_idx],
                            self.sample_rate,
                            default_view,
                            0.0,
                            &self.x_axis_link,
                            analysis_type,
                            &self.custom_sine_waves,
                            self.fft_on_peaks,
                            self.fft_smoothing,
                        );
                        widgets.push(widget);
                    }
                }

                self.position_input = "0.0".to_string();
                self.view_window_input = format!("{:.1}", default_view);
                self.view_duration = default_view;
                self.time_offset = 0.0;
                self.show_channel_selector = false;

                self.screen = Screen::Analytics { widgets, analysis_type };
            }
            Message::SwitchToViewing => {
                let default_view = 5.0_f64.min(self.total_duration);
                self.x_axis_link = AxisLink::new();
                let widgets = build_zoomed_widgets_data(
                    &self.channels,
                    &self.channel_selected,
                    self.sample_rate,
                    &self.channel_names,
                    &self.channel_units,
                    default_view,
                    0.0,
                    &self.x_axis_link,
                );

                self.position_input = "0.0".to_string();
                self.view_window_input = format!("{:.1}", default_view);
                self.view_duration = default_view;
                self.time_offset = 0.0;
                self.show_channel_selector = false;

                self.screen = Screen::Viewing { widgets };
            }
            Message::DetectPeaks => {
                if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                    // Rebuild all analysis plots
                    let mut new_widgets = Vec::new();
                    let mut ch_idx_vec = Vec::new();
                    for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                        if selected {
                            ch_idx_vec.push(ch_idx);
                        }
                    }

                    for ch_idx in ch_idx_vec {
                        let widget = build_analysis_plot(
                            &self.channels[ch_idx],
                            &self.channel_names[ch_idx],
                            &self.channel_units[ch_idx],
                            self.sample_rate,
                            self.view_duration,
                            self.time_offset,
                            &self.x_axis_link,
                            *analysis_type,
                            &self.custom_sine_waves,
                            self.fft_on_peaks,
                            self.fft_smoothing,
                        );
                        new_widgets.push(widget);
                    }
                    *widgets = new_widgets;
                }
            }
            Message::SelectAnalysisType(atype) => {
                if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                    *analysis_type = atype;
                    // Rebuild all analysis plots with new analysis type
                    let mut new_widgets = Vec::new();
                    let mut ch_idx_vec = Vec::new();
                    for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                        if selected {
                            ch_idx_vec.push(ch_idx);
                        }
                    }

                    for ch_idx in ch_idx_vec {
                        let widget = build_analysis_plot(
                            &self.channels[ch_idx],
                            &self.channel_names[ch_idx],
                            &self.channel_units[ch_idx],
                            self.sample_rate,
                            self.view_duration,
                            self.time_offset,
                            &self.x_axis_link,
                            *analysis_type,
                            &self.custom_sine_waves,
                            self.fft_on_peaks,
                            self.fft_smoothing,
                        );
                        new_widgets.push(widget);
                    }
                    *widgets = new_widgets;
                }
            }
            Message::ToggleCustomSineWaves => {
                self.show_custom_sine_waves = !self.show_custom_sine_waves;
                // Rebuild reconstruction if showing waves
                if self.show_custom_sine_waves && !self.custom_sine_waves.is_empty() {
                    // Find first selected channel
                    if let Some(first_ch_idx) = self.channel_selected.iter().position(|&b| b) {
                        // Calculate mean of detected peaks to initialize amplitude
                        let start_idx = (self.time_offset * self.sample_rate).floor() as usize;
                        let end_idx = ((self.time_offset + self.view_duration) * self.sample_rate).ceil() as usize;
                        let start_idx = start_idx.min(self.channels[first_ch_idx].len());
                        let end_idx = end_idx.min(self.channels[first_ch_idx].len());

                        let peaks = detect_peaks_simple(&self.channels[first_ch_idx][start_idx..end_idx], self.sample_rate);
                        if !peaks.is_empty() {
                            let mean_peak = peaks.iter().map(|p| p[1]).sum::<f64>() / peaks.len() as f64;
                            self.reconstruction_offset = mean_peak as f32;
                            self.reconstruction_offset_input = format!("{:.4}", mean_peak);
                        }

                        self.reconstruction_widget = Some(build_reconstruction_plot_with_offset(
                            &self.custom_sine_waves,
                            &self.channels[first_ch_idx],
                            self.sample_rate,
                            self.view_duration,
                            self.time_offset,
                            self.reconstruction_offset,
                            self.fft_on_peaks,
                        ));
                    }
                } else {
                    self.reconstruction_widget = None;
                }
                if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                    if *analysis_type == AnalysisType::FourierTransform {
                        let mut new_widgets = Vec::new();
                        for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                            if selected {
                                let widget = build_analysis_plot(
                                    &self.channels[ch_idx],
                                    &self.channel_names[ch_idx],
                                    &self.channel_units[ch_idx],
                                    self.sample_rate,
                                    self.view_duration,
                                    self.time_offset,
                                    &self.x_axis_link,
                                    *analysis_type,
                                    &self.custom_sine_waves,
                                    self.fft_on_peaks,
                                    self.fft_smoothing,
                                );
                                new_widgets.push(widget);
                            }
                        }
                        *widgets = new_widgets;
                    }
                }
            }
            Message::SetCustomFrequencyInput(input) => {
                self.custom_frequency_input = input;
            }
            Message::SetCustomAmplitudeInput(input) => {
                self.custom_amplitude_input = input;
            }
            Message::SetCustomPhaseInput(input) => {
                self.custom_phase_input = input;
            }
            Message::AddCustomSineWave => {
                if let (Ok(freq), Ok(amp), Ok(phase)) = (
                    self.custom_frequency_input.parse::<f64>(),
                    self.custom_amplitude_input.parse::<f64>(),
                    self.custom_phase_input.parse::<f64>(),
                ) {
                    if freq >= 0.0001 && amp > 0.0 {
                        self.custom_sine_waves.push((freq, amp, phase));
                        // Rebuild reconstruction widget
                        if self.show_custom_sine_waves {
                            // Find first selected channel
                            if let Some(first_ch_idx) = self.channel_selected.iter().position(|&b| b) {
                                // Initialize amplitude from mean of peaks if not already set
                                if self.reconstruction_offset == 0.0 {
                                    let start_idx = (self.time_offset * self.sample_rate).floor() as usize;
                                    let end_idx = ((self.time_offset + self.view_duration) * self.sample_rate).ceil() as usize;
                                    let start_idx = start_idx.min(self.channels[first_ch_idx].len());
                                    let end_idx = end_idx.min(self.channels[first_ch_idx].len());

                                    let peaks = detect_peaks_simple(&self.channels[first_ch_idx][start_idx..end_idx], self.sample_rate);
                                    if !peaks.is_empty() {
                                        let mean_peak = peaks.iter().map(|p| p[1]).sum::<f64>() / peaks.len() as f64;
                                        self.reconstruction_offset = mean_peak as f32;
                                        self.reconstruction_offset_input = format!("{:.4}", mean_peak);
                                    }
                                }

                                self.reconstruction_widget = Some(build_reconstruction_plot_with_offset(
                                    &self.custom_sine_waves,
                                    &self.channels[first_ch_idx],
                                    self.sample_rate,
                                    self.view_duration,
                                    self.time_offset,
                                    self.reconstruction_offset,
                                    self.fft_on_peaks,
                                ));
                            }
                        }
                        // Trigger FFT plots to redraw with new sine wave
                        if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                            if *analysis_type == AnalysisType::FourierTransform {
                                let mut new_widgets = Vec::new();
                                for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                                    if selected {
                                        let widget = build_analysis_plot(
                                            &self.channels[ch_idx],
                                            &self.channel_names[ch_idx],
                                            &self.channel_units[ch_idx],
                                            self.sample_rate,
                                            self.view_duration,
                                            self.time_offset,
                                            &self.x_axis_link,
                                            *analysis_type,
                                            &self.custom_sine_waves,
                                            self.fft_on_peaks,
                                            self.fft_smoothing,
                                        );
                                        new_widgets.push(widget);
                                    }
                                }
                                *widgets = new_widgets;
                            }
                        }
                    }
                }
            }
            Message::RemoveCustomSineWave(idx) => {
                if idx < self.custom_sine_waves.len() {
                    self.custom_sine_waves.remove(idx);
                    // Rebuild reconstruction widget
                    if self.show_custom_sine_waves {
                        // Find first selected channel
                        if let Some(first_ch_idx) = self.channel_selected.iter().position(|&b| b) {
                            self.reconstruction_widget = Some(build_reconstruction_plot_with_offset(
                                &self.custom_sine_waves,
                                &self.channels[first_ch_idx],
                                self.sample_rate,
                                self.view_duration,
                                self.time_offset,
                                self.reconstruction_offset,
                                self.fft_on_peaks,
                            ));
                        }
                    }
                    // Trigger FFT plots to redraw
                    if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                        if *analysis_type == AnalysisType::FourierTransform {
                            let mut new_widgets = Vec::new();
                            for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                                if selected {
                                    let widget = build_analysis_plot(
                                        &self.channels[ch_idx],
                                        &self.channel_names[ch_idx],
                                        &self.channel_units[ch_idx],
                                        self.sample_rate,
                                        self.view_duration,
                                        self.time_offset,
                                        &self.x_axis_link,
                                        *analysis_type,
                                        &self.custom_sine_waves,
                                        self.fft_on_peaks,
                                        self.fft_smoothing,
                                    );
                                    new_widgets.push(widget);
                                }
                            }
                            *widgets = new_widgets;
                        }
                    }
                }
            }
            Message::SetReconstructionOffsetSlider(input) => {
                self.reconstruction_offset_input = input.clone();
                // Parse and update offset
                if let Ok(offset) = input.parse::<f32>() {
                    self.reconstruction_offset = offset;

                    // Rebuild reconstruction widget with new offset
                    if let Screen::Analytics { .. } = &self.screen {
                        if !self.custom_sine_waves.is_empty() && !self.channels.is_empty() {
                            self.reconstruction_widget = Some(build_reconstruction_plot_with_offset(
                                &self.custom_sine_waves,
                                &self.channels[0],
                                self.sample_rate,
                                self.reconstruction_view_duration,
                                self.reconstruction_time_offset,
                                self.reconstruction_offset,
                                self.fft_on_peaks,
                            ));
                        }
                    }
                }
            }
            Message::ToggleFFTOnPeaks => {
                self.fft_on_peaks = !self.fft_on_peaks;
                // Trigger FFT plots to redraw and rebuild reconstruction widget
                if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                    if *analysis_type == AnalysisType::FourierTransform {
                        let mut new_widgets = Vec::new();
                        for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                            if selected {
                                let widget = build_analysis_plot(
                                    &self.channels[ch_idx],
                                    &self.channel_names[ch_idx],
                                    &self.channel_units[ch_idx],
                                    self.sample_rate,
                                    self.view_duration,
                                    self.time_offset,
                                    &self.x_axis_link,
                                    *analysis_type,
                                    &self.custom_sine_waves,
                                    self.fft_on_peaks,
                                    self.fft_smoothing,
                                );
                                new_widgets.push(widget);
                            }
                        }
                        *widgets = new_widgets;
                    }
                }
                // Also rebuild reconstruction widget if it exists
                if self.show_custom_sine_waves && !self.custom_sine_waves.is_empty() {
                    if let Some(first_ch_idx) = self.channel_selected.iter().position(|&b| b) {
                        self.reconstruction_widget = Some(build_reconstruction_plot_with_offset(
                            &self.custom_sine_waves,
                            &self.channels[first_ch_idx],
                            self.sample_rate,
                            self.view_duration,
                            self.time_offset,
                            self.reconstruction_offset,
                            self.fft_on_peaks,
                        ));
                    }
                }
            }
            Message::ToggleFFTSmoothing => {
                self.fft_smoothing = !self.fft_smoothing;
                if let Screen::Analytics { widgets, analysis_type } = &mut self.screen {
                    if *analysis_type == AnalysisType::FourierTransform {
                        let mut new_widgets = Vec::new();
                        for (ch_idx, &selected) in self.channel_selected.iter().enumerate() {
                            if selected {
                                let widget = build_analysis_plot(
                                    &self.channels[ch_idx],
                                    &self.channel_names[ch_idx],
                                    &self.channel_units[ch_idx],
                                    self.sample_rate,
                                    self.view_duration,
                                    self.time_offset,
                                    &self.x_axis_link,
                                    *analysis_type,
                                    &self.custom_sine_waves,
                                    self.fft_on_peaks,
                                    self.fft_smoothing,
                                );
                                new_widgets.push(widget);
                            }
                        }
                        *widgets = new_widgets;
                    }
                }
            }
            Message::OpenFileDialog => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("ABF Files", &["abf"])
                    .add_filter("All Files", &["*"])
                    .pick_file()
                {
                    if let Some(path_str) = path.to_str() {
                        self.selected_file_path = path_str.to_string();
                    }
                }
            }
            Message::SetReconstructionPositionInput(input) => {
                self.reconstruction_position_input = input;
            }
            Message::SetReconstructionViewWindowInput(input) => {
                self.reconstruction_view_window_input = input;
            }
            Message::UpdateReconstructionViewSettings => {
                if let Ok(offset) = self.reconstruction_position_input.parse::<f64>() {
                    self.reconstruction_time_offset = offset.max(0.0);
                }
                if let Ok(window) = self.reconstruction_view_window_input.parse::<f64>() {
                    self.reconstruction_view_duration = window.max(0.1);
                }

                // Rebuild reconstruction widget with new settings
                if let Screen::Analytics { .. } = &self.screen {
                    if !self.custom_sine_waves.is_empty() && !self.channels.is_empty() {
                        self.reconstruction_widget = Some(build_reconstruction_plot_with_offset(
                            &self.custom_sine_waves,
                            &self.channels[0],
                            self.sample_rate,
                            self.reconstruction_view_duration,
                            self.reconstruction_time_offset,
                            self.reconstruction_offset,
                            self.fft_on_peaks,
                        ));
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Selecting => self.view_selecting(),
            Screen::Viewing { .. } => self.view_viewing(),
            Screen::Analytics { .. } => self.view_analytics(),
        }
    }

    fn view_selecting(&self) -> Element<'_, Message> {
        let title = text("ABF Analytics Viewer").size(32);

        // File selection section
        let file_section = {
            column![
                text("Select ABF File:").size(14),
                row![
                    container(text(&self.selected_file_path)).width(Fill).padding(8),
                    button("Browse...").on_press(Message::OpenFileDialog).padding(8),
                ]
                .spacing(8)
                .width(Fill),
            ]
            .spacing(8)
            .padding(20)
        };

        let channels_selector = {
            let mut col = column![text("Select Channels to Load:").size(18)].spacing(12).padding(20);
            for (idx, selected) in self.channel_selected.iter().enumerate() {
                let label = format!("Channel {}", idx);
                col = col.push(checkbox(*selected).label(label).on_toggle(move |_| Message::ToggleChannel(idx)));
            }
            col
        };

        let content = column![
            container(title).padding(20).width(Fill),
            file_section,
            scrollable(container(channels_selector).padding(10).width(Fill)).height(Fill),
            container(button("Load Data & View Signals").on_press(Message::ConfirmSelection).padding(12))
                .padding(20)
                .width(Fill),
        ]
        .spacing(0)
        .height(Fill);

        container(content).height(Fill).width(Fill).center_x(Fill).center_y(Fill).into()
    }

    fn view_viewing(&self) -> Element<'_, Message> {
        if let Screen::Viewing { widgets } = &self.screen {
            let selected_count = self.channel_selected.iter().filter(|&&b| b).count();

            // Control panel
            let controls = self.build_control_panel(
                &self.position_input,
                &self.view_window_input,
                false,
                Some(selected_count),
                self.show_channel_selector,
            );

            // Height slider
            let height_control = row![
                text("Graph Height:").size(11).width(90),
                slider(50.0..=500.0, self.graph_height, Message::SetGraphHeight).width(200).step(5.0),
                text(format!("{:.0}px", self.graph_height)).size(11).width(50),
            ]
            .spacing(10)
            .padding(10)
            .align_y(Center);

            // Plots
            let mut plots_col = column![].spacing(2);
            for (index, widget) in widgets.iter().enumerate() {
                let plot = widget.view().map(move |msg| Message::PlotMessage(index, msg));
                let ch_label = text(format!("Channel {index}")).size(12);
                let row_layout = row![
                    container(ch_label).width(60).padding(5),
                    container(plot).height(self.graph_height).width(Fill)
                ]
                .spacing(0)
                .width(Fill);
                plots_col = plots_col.push(row_layout);
            }

            // Channel selector popup overlay
            let content = if self.show_channel_selector {
                column![
                    controls,
                    height_control,
                    container(self.channel_selector_popup()).padding(20),
                    scrollable(plots_col).height(Fill)
                ]
            } else {
                column![controls, height_control, scrollable(plots_col).height(Fill),]
            };

            container(content).height(Fill).width(Fill).padding(0).into()
        } else {
            text("Error").into()
        }
    }

    fn view_analytics(&self) -> Element<'_, Message> {
        if let Screen::Analytics { widgets, analysis_type } = &self.screen {
            let selected_count = self.channel_selected.iter().filter(|&&b| b).count();

            let mut controls = self
                .build_control_panel(
                    &self.position_input,
                    &self.view_window_input,
                    true,
                    Some(selected_count),
                    self.show_channel_selector,
                )
                .push(
                    row![
                        text("Analysis Type:").size(11).width(60),
                        pick_list(AnalysisType::all(), Some(*analysis_type), Message::SelectAnalysisType,).width(Fill),
                    ]
                    .spacing(10)
                    .align_y(Center),
                );

            // Add custom sine wave checkbox and FFT checkbox only for FFT
            if matches!(*analysis_type, AnalysisType::FourierTransform) {
                controls = controls.push(
                    row![
                        checkbox(self.show_custom_sine_waves)
                            .label("Overlay Custom Sine Waves")
                            .on_toggle(|_| Message::ToggleCustomSineWaves),
                        checkbox(self.fft_on_peaks)
                            .label("FFT on Detected Peaks Only")
                            .on_toggle(|_| Message::ToggleFFTOnPeaks),
                        checkbox(self.fft_smoothing)
                            .label("FFT Smoothing")
                            .on_toggle(|_| Message::ToggleFFTSmoothing),
                        space().width(Fill),
                    ]
                    .spacing(20)
                    .padding(10),
                );
            }

            // Add frequency/amplitude/phase input only when custom sine waves are enabled
            if *analysis_type == AnalysisType::FourierTransform && self.show_custom_sine_waves {
                controls = controls.push(
                    row![
                        column![
                            text("Frequency (Hz)").size(10),
                            text_input("10.0", &self.custom_frequency_input)
                                .on_input(Message::SetCustomFrequencyInput)
                                .width(80)
                                .padding(4),
                        ]
                        .spacing(4),
                        column![
                            text("Amplitude").size(10),
                            text_input("1.0", &self.custom_amplitude_input)
                                .on_input(Message::SetCustomAmplitudeInput)
                                .width(80)
                                .padding(4),
                        ]
                        .spacing(4),
                        column![
                            text("Phase (rad)").size(10),
                            text_input("0.0", &self.custom_phase_input)
                                .on_input(Message::SetCustomPhaseInput)
                                .width(80)
                                .padding(4),
                        ]
                        .spacing(4),
                        button("Add Wave").on_press(Message::AddCustomSineWave).padding(6),
                        space().width(Fill),
                    ]
                    .spacing(8)
                    .align_y(Center)
                    .padding(10),
                );

                // List of custom waves
                if !self.custom_sine_waves.is_empty() {
                    let mut waves_list = column![text("Custom Waves:").size(10)].spacing(4).padding(10);
                    for (idx, (freq, amp, phase)) in self.custom_sine_waves.iter().enumerate() {
                        waves_list = waves_list.push(
                            row![
                                text(format!("{:.1}Hz @ {:.2}A φ={:.2}", freq, amp, phase)).size(10),
                                space().width(Fill),
                                button("×").on_press(Message::RemoveCustomSineWave(idx)).padding(2),
                            ]
                            .spacing(4)
                            .width(Fill),
                        );
                    }
                    controls = controls.push(waves_list);

                    // Reconstruction sine offset input
                    controls = controls.push(
                        row![
                            column![
                                text("Sine Offset").size(10),
                                text_input("0.0", &self.reconstruction_offset_input)
                                    .on_input(Message::SetReconstructionOffsetSlider)
                                    .width(100)
                                    .padding(4),
                            ]
                            .spacing(4),
                            space().width(Fill),
                        ]
                        .spacing(8)
                        .align_y(Center)
                        .padding(10),
                    );

                    // Reconstruction view settings
                    controls = controls.push(
                        row![
                            column![
                                text("Recon Position (s)").size(10),
                                text_input("0.0", &self.reconstruction_position_input)
                                    .on_input(Message::SetReconstructionPositionInput)
                                    .width(100)
                                    .padding(4),
                            ]
                            .spacing(4),
                            column![
                                text("Recon Window (s)").size(10),
                                text_input("5.0", &self.reconstruction_view_window_input)
                                    .on_input(Message::SetReconstructionViewWindowInput)
                                    .width(100)
                                    .padding(4),
                            ]
                            .spacing(4),
                            button("Update").on_press(Message::UpdateReconstructionViewSettings).padding(6),
                            space().width(Fill),
                        ]
                        .spacing(8)
                        .align_y(Center)
                        .padding(10),
                    );
                }
            }

            let controls = controls;

            // Height slider
            let height_control = row![
                text("Graph Height:").size(11).width(90),
                slider(50.0..=300.0, self.graph_height, Message::SetGraphHeight).width(200).step(5.0),
                text(format!("{:.0}px", self.graph_height)).size(11).width(50),
            ]
            .spacing(10)
            .padding(10)
            .align_y(Center);

            // Display analysis plots for all selected channels
            let mut plots_col = column![].spacing(2);
            let mut ch_idx = 0;
            for (index, &selected) in self.channel_selected.iter().enumerate() {
                if selected && ch_idx < widgets.len() {
                    let widget = &widgets[ch_idx];
                    let plot = widget.view().map(move |msg| Message::PlotMessage(ch_idx, msg));
                    let ch_label = text(format!("Channel {}", index)).size(12);
                    let row_layout = row![
                        container(ch_label).width(60).padding(5),
                        container(plot).height(self.graph_height).width(Fill).padding(8)
                    ]
                    .spacing(0)
                    .width(Fill);
                    plots_col = plots_col.push(row_layout);
                    ch_idx += 1;
                }
            }

            // Add reconstruction plot if FFT mode with custom sine waves
            if matches!(*analysis_type, AnalysisType::FourierTransform) && self.show_custom_sine_waves && !self.custom_sine_waves.is_empty() {
                if let Some(ref reconstruction_widget) = self.reconstruction_widget {
                    // Reconstruction plot: map messages to idempotent message (no effect)
                    let recon_plot = reconstruction_widget.view().map(|_msg| Message::SetGraphHeight(self.graph_height));
                    let recon_label = text("Reconstructed Signal").size(12);
                    let recon_row = row![
                        container(recon_label).width(60).padding(5),
                        container(recon_plot).height(self.graph_height).width(Fill).padding(8)
                    ]
                    .spacing(0)
                    .width(Fill);
                    plots_col = plots_col.push(recon_row);
                }
            }

            let content = if self.show_channel_selector {
                column![
                    controls,
                    height_control,
                    container(self.channel_selector_popup()).padding(20),
                    scrollable(plots_col).height(Fill)
                ]
            } else {
                column![controls, height_control, scrollable(plots_col).height(Fill)]
            };

            container(content).height(Fill).width(Fill).padding(0).into()
        } else {
            text("Error").into()
        }
    }

    fn build_control_panel(
        &self,
        position_input: &str,
        view_window_input: &str,
        is_analytics: bool,
        selected_count: Option<usize>,
        _show_selector: bool,
    ) -> iced::widget::Column<'static, Message> {
        let title = if is_analytics {
            "Analytics - Channel Analysis".to_string()
        } else {
            format!("Signal Viewer - {} Ch", selected_count.unwrap_or(0))
        };

        let mut controls = column![text(title).size(20)].spacing(12).padding(15);

        // Position & Window inputs in a row
        let input_row = row![
            column![
                text("Position (s)").size(10),
                text_input("0.0", position_input)
                    .on_input(Message::SetPositionInput)
                    .width(100)
                    .padding(6),
            ]
            .spacing(4),
            column![
                text("Window (s)").size(10),
                text_input("5.0", view_window_input)
                    .on_input(Message::SetViewWindowInput)
                    .width(100)
                    .padding(6),
            ]
            .spacing(4),
            space().width(Fill),
        ]
        .spacing(12)
        .align_y(Center);

        controls = controls.push(input_row);

        // Action buttons
        let mut button_row = row![
            button("<< Pan").on_press(Message::PanLeft).padding(8),
            button("Pan >>").on_press(Message::PanRight).padding(8),
            button("Update").on_press(Message::UpdateViewSettings).padding(8),
            button("Reset Zoom").on_press(Message::ResetZoom).padding(8),
            button("Channels...").on_press(Message::ToggleChannelSelector).padding(8),
        ]
        .spacing(8);

        if !is_analytics {
            button_row = button_row.push(button("Analysis >>").on_press(Message::SwitchToAnalytics).padding(8));
        } else {
            button_row = button_row.push(button("<< Back").on_press(Message::SwitchToViewing).padding(8));
            button_row = button_row.push(button("Detect").on_press(Message::DetectPeaks).padding(8));
        }

        controls.push(button_row)
    }

    fn channel_selector_popup(&self) -> Element<'static, Message> {
        let mut col = column![text("Select Channels:").size(16)].spacing(8).padding(15);

        for (idx, selected) in self.channel_selected_temp.iter().enumerate() {
            let label = format!("Channel {idx}");
            col = col.push(checkbox(*selected).label(label).on_toggle(move |_| Message::ToggleChannelTemp(idx)));
        }

        let button_row = row![
            button("Confirm").on_press(Message::ConfirmChannelSelection).padding(6),
            button("Cancel").on_press(Message::CancelChannelSelection).padding(6),
        ]
        .spacing(8);

        col.push(button_row).into()
    }
}

fn build_zoomed_widgets_data(
    channels: &[Vec<f32>],
    channel_selected: &[bool; 9],
    sample_rate: f64,
    channel_names: &[String],
    channel_units: &[String],
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
                for (min_pt, max_pt) in buckets.into_iter().flatten() {
                    // Add points in time order to maintain waveform shape
                    if min_pt[0] < max_pt[0] {
                        downsampled.push(min_pt);
                        downsampled.push(max_pt);
                    } else {
                        downsampled.push(max_pt);
                        downsampled.push(min_pt);
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
                .with_label(ch_name)
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

fn read_channel_data(
    file_path: &str,
    _channel_selected: [bool; 9],
    _start_offset: f64,
    _duration: f64,
) -> (Vec<Vec<f32>>, f64, Vec<String>, Vec<String>, f64) {
    let mut channels = Vec::new();
    let mut sample_rate = 1.0;
    let mut channel_units: Vec<String> = Vec::new();
    let mut channel_names: Vec<String> = Vec::new();
    let mut total_duration = 0.0;

    if let Ok(mut reader) = AbfReader::open_with_options(
        Path::new(file_path),
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
        let start = (i + 1).saturating_sub(window_size);
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

fn smooth_fft_magnitude(points: &[[f64; 2]], window_size: usize) -> Vec<[f64; 2]> {
    if points.is_empty() || window_size <= 1 {
        return points.to_vec();
    }

    let mut out = Vec::with_capacity(points.len());
    let half = window_size / 2;

    for i in 0..points.len() {
        let start = i.saturating_sub(half);
        let end = (i + half + 1).min(points.len());
        let mut sum = 0.0_f64;
        for point in points.iter().take(end).skip(start) {
            sum += point[1];
        }
        let avg = sum / (end - start) as f64;
        out.push([points[i][0], avg]);
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
    sample_rate: f64,
    view_duration: f64,
    time_offset: f64,
    x_axis_link: &AxisLink,
    analysis_type: AnalysisType,
    custom_sine_waves: &[(f64, f64, f64)],
    fft_on_peaks: bool,
    fft_smoothing: bool,
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
        for (min_pt, max_pt) in buckets.into_iter().flatten() {
            if min_pt[0] < max_pt[0] {
                downsampled.push(min_pt);
                downsampled.push(max_pt);
            } else {
                downsampled.push(max_pt);
                downsampled.push(min_pt);
            }
        }

        filtered_positions = downsampled;
    }

    let signal_series = Series::line_only(filtered_positions.clone(), LineStyle::Solid)
        .with_label(channel_name.to_string())
        .with_color(Color::from_rgb(0.3, 0.3, 0.9));

    // Build plot with signal
    let mut builder = PlotWidgetBuilder::new().add_series(signal_series);

    // Filter and add peaks only if PeakDetection analysis type
    if matches!(analysis_type, AnalysisType::PeakDetection) {
        // Detect on full channel so peak timing stays in absolute time, then filter to visible window.
        let all_peaks = detect_peaks_simple(channel_data, sample_rate);
        let filtered_peaks: Vec<[f64; 2]> = all_peaks
            .iter()
            .filter(|p| p[0] >= time_offset && p[0] <= zoomed_x_max)
            .copied()
            .collect();

        if !filtered_peaks.is_empty() {
            let mut peak_markers: Vec<[f64; 2]> = Vec::new();
            for peak in &filtered_peaks {
                peak_markers.push(*peak);
            }

            let peaks_series = Series::line_only(peak_markers, LineStyle::Solid)
                .with_label("Peaks")
                .with_color(Color::from_rgb(1.0, 0.2, 0.2));
            builder = builder.add_series(peaks_series);
        }
    }

    if matches!(analysis_type, AnalysisType::FourierTransform) {
        use rustfft::{num_complex::Complex, FftPlanner};
        use std::f64::consts::PI;

        let start_idx = (time_offset * sample_rate).floor() as usize;
        let end_idx = ((time_offset + view_duration) * sample_rate).ceil() as usize;
        let start_idx = start_idx.min(channel_data.len());
        let end_idx = end_idx.min(channel_data.len());

        // Determine which data to use for FFT
        let mut peak_effective_sample_rate = sample_rate;
        let fft_data: Vec<f32> = if fft_on_peaks {
            // Detect peaks in full signal and filter by current visible window.
            // This preserves absolute peak timing and gives more stable effective sampling.
            let all_peaks = detect_peaks_simple(channel_data, sample_rate);
            let visible_peaks: Vec<[f64; 2]> = all_peaks
                .iter()
                .filter(|p| p[0] >= time_offset && p[0] <= zoomed_x_max)
                .copied()
                .collect();

            if visible_peaks.len() >= 2 {
                let first_t = visible_peaks.first().map(|p| p[0]).unwrap_or(time_offset);
                let last_t = visible_peaks.last().map(|p| p[0]).unwrap_or(zoomed_x_max);
                let span = (last_t - first_t).max(1e-9);
                peak_effective_sample_rate = (visible_peaks.len() - 1) as f64 / span;
            } else if view_duration > 0.0 {
                peak_effective_sample_rate = (visible_peaks.len().max(1) as f64) / view_duration;
            }

            visible_peaks.iter().map(|p| p[1] as f32).collect()
        } else {
            // Use raw signal data
            channel_data[start_idx..end_idx].to_vec()
        };

        let n = fft_data.len();

        if n == 0 || (fft_on_peaks && n < 4) {
            // Fallthrough: no data to FFT
        } else {
            // First, remove DC offset by subtracting the mean
            let mean: f64 = fft_data.iter().map(|&x| x as f64).sum::<f64>() / n as f64;

            // FFT length: zero-pad to next power of two for speed
            let fft_len = n.next_power_of_two();
            let mut buffer = vec![Complex::new(0.0, 0.0); fft_len];

            // Hann window and compute window sum for amplitude correction
            let mut window_sum = 0.0_f64;
            if n > 1 {
                for i in 0..n {
                    let w = 0.5 * (1.0 - (2.0 * PI * i as f64 / (n as f64 - 1.0)).cos());
                    // Apply window to mean-removed signal
                    let sample = fft_data[i] as f64 - mean;
                    buffer[i].re = sample * w;
                    window_sum += w;
                }
            } else {
                buffer[0].re = (fft_data[0] as f64) - mean;
                window_sum = 1.0;
            }

            // FFT
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(fft_len);
            fft.process(&mut buffer);

            // Scale: correct for window gain and produce single-sided amplitude
            let scale = if window_sum > 0.0 { 2.0 / window_sum } else { 2.0 / (n as f64) };
            let half = fft_len / 2;
            let mut fft_markers: Vec<[f64; 2]> = Vec::with_capacity(half);
            // Now we can safely include all frequency bins including 0 Hz (DC will be minimal)
            for i in 0..half {
                let freq = if fft_on_peaks {
                    i as f64 * peak_effective_sample_rate / fft_len as f64
                } else {
                    i as f64 * sample_rate / fft_len as f64
                };
                let mag = buffer[i].norm() * scale;
                fft_markers.push([freq, mag]);
            }

            if fft_smoothing && fft_markers.len() >= 5 {
                fft_markers = smooth_fft_magnitude(&fft_markers, 5);
            }

            if !fft_markers.is_empty() {
                // Use different label and color if FFT is on peaks
                let (fft_label, fft_color) = if fft_on_peaks {
                    ("Peak Detection FFT", Color::from_rgb(0.8, 0.4, 0.2))
                } else {
                    ("FFT Magnitude", Color::from_rgb(0.2, 0.4, 1.0))
                };

                let fft_series = Series::line_only(fft_markers.clone(), LineStyle::Solid)
                    .with_label(fft_label)
                    .with_color(fft_color);

                let y_min = fft_markers.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
                let y_max = fft_markers.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
                let y_pad = if y_max > y_min {
                    (y_max - y_min) * 0.1
                } else {
                    y_max.abs().max(1.0) * 0.1
                };

                // Build FFT-only plot (do not reuse time-domain builder settings)
                let mut builder = PlotWidgetBuilder::new()
                    .add_series(fft_series)
                    .with_cursor_overlay(true)
                    .with_x_label("Frequency (Hz)")
                    .with_y_label("Magnitude")
                    .with_y_lim(y_min - y_pad, y_max + y_pad)
                    .with_x_axis_link(x_axis_link.clone())
                    .disable_legend()
                    .disable_controls_help()
                    .with_crosshairs(true);

                // Add custom sine wave series if present
                let custom_colors = [
                    Color::from_rgb(1.0, 0.5, 0.0), // Orange
                    Color::from_rgb(1.0, 0.0, 1.0), // Magenta
                    Color::from_rgb(0.0, 1.0, 1.0), // Cyan
                    Color::from_rgb(1.0, 1.0, 0.0), // Yellow
                    Color::from_rgb(0.5, 1.0, 0.0), // Lime
                ];

                for (idx, &(freq, amp, _phase)) in custom_sine_waves.iter().enumerate() {
                    // Create a marker at the sine wave frequency with scaled amplitude
                    let sine_point = vec![[freq, amp]];
                    let color = custom_colors[idx % custom_colors.len()];
                    let series = Series::line_only(sine_point, LineStyle::Solid)
                        .with_label(format!("{}Hz", freq))
                        .with_color(color);
                    builder = builder.add_series(series);
                }

                return builder.build().expect("Failed to build FFT plot");
            }
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

#[allow(dead_code)]
fn build_reconstruction_plot(
    custom_sine_waves: &[(f64, f64, f64)],
    channel_data: &[f32],
    sample_rate: f64,
    view_duration: f64,
    time_offset: f64,
) -> PlotWidget {
    if custom_sine_waves.is_empty() {
        return PlotWidgetBuilder::new()
            .with_x_label("Time (s)")
            .with_y_label("Amplitude")
            .build()
            .expect("Failed to build reconstruction plot");
    }

    // Generate time-domain reconstruction
    let num_samples = (view_duration * sample_rate) as usize;
    if num_samples == 0 {
        return PlotWidgetBuilder::new()
            .with_x_label("Time (s)")
            .with_y_label("Amplitude")
            .build()
            .expect("Failed to build reconstruction plot");
    }

    // Generate combined sine wave by summing all components
    let mut reconstruction: Vec<f64> = vec![0.0; num_samples];
    for &(freq, amp, phase) in custom_sine_waves.iter() {
        for i in 0..num_samples {
            let time = time_offset + (i as f64) / sample_rate;
            let sample = amp * (2.0 * std::f64::consts::PI * freq * time + phase).sin();
            reconstruction[i] += sample;
        }
    }

    // Convert reconstruction to plot points
    let mut recon_points: Vec<[f64; 2]> = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let time = time_offset + (i as f64) / sample_rate;
        recon_points.push([time, reconstruction[i]]);
    }

    // Extract raw signal points in the same time window
    let start_idx = (time_offset * sample_rate).floor() as usize;
    let end_idx = ((time_offset + view_duration) * sample_rate).ceil() as usize;
    let start_idx = start_idx.min(channel_data.len());
    let end_idx = end_idx.min(channel_data.len());

    let mut raw_points: Vec<[f64; 2]> = Vec::new();
    for (i, &value) in channel_data[start_idx..end_idx].iter().enumerate() {
        let time = time_offset + (i as f64) / sample_rate;
        raw_points.push([time, value as f64]);
    }

    // Calculate Y bounds from both signals
    let mut all_values = reconstruction.clone();
    all_values.extend(channel_data[start_idx..end_idx].iter().map(|&v| v as f64));
    let y_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_range = y_max - y_min;
    let padding = if y_range > 0.0 { y_range * 0.1 } else { 0.5 };

    // Raw signal series (blue)
    let raw_series = Series::line_only(raw_points, LineStyle::Solid)
        .with_label("Original Signal")
        .with_color(Color::from_rgb(0.2, 0.5, 0.9));

    // Reconstructed signal series (red)
    let recon_series = Series::line_only(recon_points, LineStyle::Solid)
        .with_label("Reconstructed Signal")
        .with_color(Color::from_rgb(0.9, 0.2, 0.2));

    PlotWidgetBuilder::new()
        .add_series(raw_series)
        .add_series(recon_series)
        .with_cursor_overlay(true)
        .with_x_label("Time (s)")
        .with_y_label("Amplitude")
        .with_y_lim(y_min - padding, y_max + padding)
        .with_crosshairs(true)
        .build()
        .expect("Failed to build reconstruction plot")
}

fn build_reconstruction_plot_with_offset(
    custom_sine_waves: &[(f64, f64, f64)],
    channel_data: &[f32],
    sample_rate: f64,
    view_duration: f64,
    time_offset: f64,
    sine_offset: f32,
    fft_on_peaks: bool,
) -> PlotWidget {
    if custom_sine_waves.is_empty() {
        return PlotWidgetBuilder::new()
            .with_x_label("Time (s)")
            .with_y_label("Amplitude")
            .build()
            .expect("Failed to build reconstruction plot");
    }

    // Extract detected peaks from the ENTIRE channel (not just visible window)
    // This gives better detection context, then filter to visible window
    let all_peaks = detect_peaks_simple(channel_data, sample_rate);
    let zoomed_x_max = time_offset + view_duration;
    let visible_peaks: Vec<[f64; 2]> = all_peaks
        .iter()
        .filter(|p| p[0] >= time_offset && p[0] <= zoomed_x_max)
        .copied()
        .collect();

    // Generate time-domain reconstruction with vertical offset
    let num_samples = (view_duration * sample_rate) as usize;
    if num_samples == 0 {
        return PlotWidgetBuilder::new()
            .with_x_label("Time (s)")
            .with_y_label("Amplitude")
            .build()
            .expect("Failed to build reconstruction plot");
    }

    // Generate combined sine wave by summing all components and adding vertical offset
    let mut reconstruction: Vec<f64> = vec![0.0; num_samples];
    for &(freq, amp, phase) in custom_sine_waves.iter() {
        for i in 0..num_samples {
            let time = time_offset + (i as f64) / sample_rate;
            let sample = amp * (2.0 * std::f64::consts::PI * freq * time + phase).sin();
            reconstruction[i] += sample;
        }
    }
    for value in reconstruction.iter_mut() {
        *value += sine_offset as f64;
    }

    let mut builder = PlotWidgetBuilder::new();

    // Handle peaks display: either peaks graph or time-domain original signal
    if fft_on_peaks && !visible_peaks.is_empty() {
        // Show detected peaks as a graph (peaks are already in absolute time)
        let peak_markers: Vec<[f64; 2]> = visible_peaks.clone();

        // Peaks series (orange)
        let peaks_series = Series::line_only(peak_markers.clone(), LineStyle::Solid)
            .with_label("Detected Peaks")
            .with_color(Color::from_rgb(1.0, 0.6, 0.0)); // orange for peaks

        builder = builder.add_series(peaks_series);

        // Convert reconstruction to plot points
        let mut recon_points: Vec<[f64; 2]> = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let time = time_offset + (i as f64) / sample_rate;
            recon_points.push([time, reconstruction[i]]);
        }

        // Reconstructed signal series (red)
        let recon_series = Series::line_only(recon_points, LineStyle::Solid)
            .with_label("Sine Reconstruction")
            .with_color(Color::from_rgb(1.0, 0.2, 0.2));

        // Calculate reconstruction bounds
        let mut all_values = reconstruction.clone();
        all_values.extend(peak_markers.iter().map(|p| p[1]));
        let y_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_range = y_max - y_min;
        let padding = if y_range > 0.0 { y_range * 0.1 } else { 0.5 };

        builder = builder
            .add_series(recon_series)
            .with_x_label("Time (s)")
            .with_y_label("Amplitude")
            .with_y_lim(y_min - padding, y_max + padding);
    } else {
        // Time-domain display with original raw signal
        // Extract raw signal points in the visible window
        let mut raw_points: Vec<[f64; 2]> = Vec::new();
        let mut raw_amplitudes = Vec::new();
        for (i, &value) in channel_data.iter().enumerate() {
            let time = (i as f64) / sample_rate;
            if time >= time_offset && time <= zoomed_x_max {
                raw_points.push([time, value as f64]);
                raw_amplitudes.push(value as f64);
            }
        }

        // Convert reconstruction to plot points
        let mut recon_points: Vec<[f64; 2]> = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let time = time_offset + (i as f64) / sample_rate;
            recon_points.push([time, reconstruction[i]]);
        }

        // Calculate Y bounds from both signals
        let mut all_values = reconstruction.clone();
        all_values.extend(raw_amplitudes.iter().cloned());
        let y_min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_range = y_max - y_min;
        let padding = if y_range > 0.0 { y_range * 0.1 } else { 0.5 };

        // Original signal series (blue)
        let signal_series = Series::line_only(raw_points, LineStyle::Solid)
            .with_label("Original Signal")
            .with_color(Color::from_rgb(0.2, 0.5, 0.9));

        // Reconstructed signal series (red)
        let recon_series = Series::line_only(recon_points, LineStyle::Solid)
            .with_label("Sine Reconstruction")
            .with_color(Color::from_rgb(0.9, 0.2, 0.2));

        builder = builder
            .add_series(signal_series)
            .add_series(recon_series)
            .with_x_label("Time (s)")
            .with_y_label("Amplitude")
            .with_y_lim(y_min - padding, y_max + padding);
    }

    builder
        .with_cursor_overlay(true)
        .with_crosshairs(true)
        .build()
        .expect("Failed to build reconstruction plot")
}
