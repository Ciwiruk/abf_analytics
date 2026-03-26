use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug)]
pub struct AbfV1Header {
    /// Group 1: File ID and Size Information (offset 0)
    pub group1_file_id: Option<Group1FileIdAndSize>,
    /// Group 2: File Structure (offset 40)
    pub group2_file_structure: Option<Group2FileStructure>,
    /// Group 3: Trial Hierarchy Information (offset 118)
    pub group3_trial_hierarchy: Option<Group3TrialHierarchy>,
    /// Group 4: Display Parameters (offset 200)
    pub group4_display: Option<Group4DisplayParameters>,
    /// Group 5: Hardware Information (offset 244)
    pub group5_hardware: Option<Group5HardwareInformation>,
    /// Group 6: Environmental Information (offset 260)
    pub group6_environment: Option<Group6EnvironmentalInformation>,
    /// Group 7: Multi-channel Information (offset 378)
    pub group7_multichannel: Option<Group7MultiChannelInformation>,
    /// Group 8: Synchronous Timer Outputs (offset 1422)
    pub group8_sync_timer: Option<Group8SynchronousTimerOutputs>,
    /// Group 9: Epoch Waveform and Pulses (offset 1436)
    pub group9_epoch: Option<Group9EpochWaveformAndPulses>,
    /// Group 10: DAC Output File (offset 1620)
    pub group10_dac_output_file: Option<Group10DacOutputFile>,
    /// Group 11: Presweep Conditioning Pulse Train (offset 1718)
    pub group11_presweep: Option<Group11PresweepConditioningPulseTrain>,
    /// Group 12: Variable Parameter User List (offset 1762)
    pub group12_variable_parameter: Option<Group12VariableParameterUserList>,
    /// Group 13: Autopeak Measurement (offset 1844)
    pub group13_autopeak: Option<Group13AutopeakMeasurement>,
    /// Group 14: Channel Arithmetic (offset 1880)
    pub group14_arithmetic: Option<Group14ChannelArithmetic>,
    /// Group 15: On-line Subtraction (offset 1932)
    pub group15_online_subtraction: Option<Group15OnlineSubtraction>,
    /// Group 16: Miscellaneous Parameters (offset 1966)
    pub group16_misc: Option<Group16MiscParameters>,
    /// Extended Group 2: Extended File Structure (offset 2048)
    pub ext_group2_file_structure: Option<ExtGroup2FileStructure>,
    /// Extended Group 3: Trial Hierarchy extension (offset 2064)
    pub ext_group3_trial_hierarchy: Option<ExtGroup3TrialHierarchy>,
    /// Extended Group 7: Multi-channel extension (offset 2074)
    pub ext_group7_multichannel: Option<ExtGroup7MultiChannel>,
    /// Group 17: Train Parameters (offset 2136)
    pub group17_train: Option<Group17TrainParameters>,
    /// Extended Group 9: Epoch extension (offset 2296)
    pub ext_group9_epoch: Option<ExtGroup9Epoch>,
    /// Extended Group 10: DAC output file extension (offset 2708)
    pub ext_group10_dac_output_file: Option<ExtGroup10DacOutputFile>,
    /// Extended Group 11: Presweep extension (offset 3260)
    pub ext_group11_presweep: Option<ExtGroup11Presweep>,
    /// Extended Group 12: Variable parameter extension (offset 3360)
    pub ext_group12_variable_parameter: Option<ExtGroup12VariableParameter>,
    /// Extended Group 6: Environmental extension (offset 4512)
    pub ext_group6_environment: Option<ExtGroup6Environmental>,
    /// Group 13 (statistics): Statistics Measurements (offset 5410)
    pub group13_statistics: Option<Group13StatisticsMeasurements>,
    /// Group 18: Application Version Data (offset 5798)
    pub group18_app_version: Option<Group18ApplicationVersionData>,
    /// Group 19: LTP Protocol (offset 5814)
    pub group19_ltp: Option<Group19LtpProtocol>,
    /// Group 20: Digidata 132x Trigger Out Flag (offset 5828)
    pub group20_dd132x_trigger_out: Option<Group20Dd132xTriggerOut>,
    /// Group 21: Epoch Resistance (offset 5836)
    pub group21_epoch_resistance: Option<Group21EpochResistance>,
    /// Group 22: Alternating Episodic Mode (offset 5892)
    pub group22_alternating_episodic: Option<Group22AlternatingEpisodicMode>,
    /// Group 23: Post-processing Actions (offset 5950, optional in 6144-byte headers)
    pub group23_post_processing: Option<Group23PostProcessingActions>,
}

#[derive(Debug, Clone, Copy)]
pub struct AbfHeaderReadOptions {
    pub group1_file_id: bool,
    pub group2_file_structure: bool,
    pub group3_trial_hierarchy: bool,
    pub group4_display: bool,
    pub group5_hardware: bool,
    pub group6_environment: bool,
    pub group7_multichannel: bool,
    pub group8_sync_timer: bool,
    pub group9_epoch: bool,
    pub group10_dac_output_file: bool,
    pub group11_presweep: bool,
    pub group12_variable_parameter: bool,
    pub group13_autopeak: bool,
    pub group14_arithmetic: bool,
    pub group15_online_subtraction: bool,
    pub group16_misc: bool,
    pub ext_group2_file_structure: bool,
    pub ext_group3_trial_hierarchy: bool,
    pub ext_group7_multichannel: bool,
    pub group17_train: bool,
    pub ext_group9_epoch: bool,
    pub ext_group10_dac_output_file: bool,
    pub ext_group11_presweep: bool,
    pub ext_group12_variable_parameter: bool,
    pub ext_group6_environment: bool,
    pub group13_statistics: bool,
    pub group18_app_version: bool,
    pub group19_ltp: bool,
    pub group20_dd132x_trigger_out: bool,
    pub group21_epoch_resistance: bool,
    pub group22_alternating_episodic: bool,
    pub group23_post_processing: bool,
}

impl Default for AbfHeaderReadOptions {
    fn default() -> Self {
        Self {
            group1_file_id: true,
            group2_file_structure: true,
            group3_trial_hierarchy: true,
            group4_display: false,
            group5_hardware: false,
            group6_environment: false,
            group7_multichannel: false,
            group8_sync_timer: false,
            group9_epoch: false,
            group10_dac_output_file: false,
            group11_presweep: false,
            group12_variable_parameter: false,
            group13_autopeak: false,
            group14_arithmetic: false,
            group15_online_subtraction: false,
            group16_misc: false,
            ext_group2_file_structure: false,
            ext_group3_trial_hierarchy: false,
            ext_group7_multichannel: false,
            group17_train: false,
            ext_group9_epoch: false,
            ext_group10_dac_output_file: false,
            ext_group11_presweep: false,
            ext_group12_variable_parameter: false,
            ext_group6_environment: false,
            group13_statistics: false,
            group18_app_version: false,
            group19_ltp: false,
            group20_dd132x_trigger_out: false,
            group21_epoch_resistance: false,
            group22_alternating_episodic: false,
            group23_post_processing: false,
        }
    }
}

impl AbfHeaderReadOptions {
    pub fn all() -> Self {
        Self {
            group1_file_id: true,
            group2_file_structure: true,
            group3_trial_hierarchy: true,
            group4_display: true,
            group5_hardware: true,
            group6_environment: true,
            group7_multichannel: true,
            group8_sync_timer: true,
            group9_epoch: true,
            group10_dac_output_file: true,
            group11_presweep: true,
            group12_variable_parameter: true,
            group13_autopeak: true,
            group14_arithmetic: true,
            group15_online_subtraction: true,
            group16_misc: true,
            ext_group2_file_structure: true,
            ext_group3_trial_hierarchy: true,
            ext_group7_multichannel: true,
            group17_train: true,
            ext_group9_epoch: true,
            ext_group10_dac_output_file: true,
            ext_group11_presweep: true,
            ext_group12_variable_parameter: true,
            ext_group6_environment: true,
            group13_statistics: true,
            group18_app_version: true,
            group19_ltp: true,
            group20_dd132x_trigger_out: true,
            group21_epoch_resistance: true,
            group22_alternating_episodic: true,
            group23_post_processing: true,
        }
    }
}

#[derive(Debug)]
pub struct Group1FileIdAndSize {
    pub file_signature: String,
    pub file_version_number: f32,
    pub operation_mode: i16,
    pub actual_acq_length: i32,
    pub num_points_ignored: i16,
    pub actual_episodes: i32,
    pub file_start_date: i32,
    pub file_start_time: i32,
    pub stopwatch_time: i32,
    pub header_version_number: f32,
    pub file_type: i16,
    pub ms_bin_format: i16,
}

#[derive(Debug)]
pub struct Group2FileStructure {
    pub data_section_ptr: i32,
    pub tag_section_ptr: i32,
    pub num_tag_entries: i32,
    pub scope_config_ptr: i32,
    pub num_scopes: i32,
    pub dac_file_ptr_legacy: i32,
    pub dac_file_num_episodes_legacy: i32,
    pub unused68: [u8; 4],
    pub delta_array_ptr: i32,
    pub num_deltas: i32,
    pub voice_tag_ptr: i32,
    pub voice_tag_entries: i32,
    pub unused88: i32,
    pub synch_array_ptr: i32,
    pub synch_array_size: i32,
    pub data_format: i16,
    pub simultaneous_scan: i16,
    pub statistics_config_ptr: i32,
    pub annotation_section_ptr: i32,
    pub num_annotations: i32,
    pub unused004: [u8; 2],
}

#[derive(Debug)]
pub struct Group3TrialHierarchy {
    pub channel_count_acquired: i16,
    pub adc_num_channels: i16,
    pub adc_sample_interval: f32,
    pub adc_second_sample_interval: f32,
    pub synch_time_unit: f32,
    pub seconds_per_run: f32,
    pub num_samples_per_episode: i32,
    pub pre_trigger_samples: i32,
    pub episodes_per_run: i32,
    pub runs_per_trial: i32,
    pub number_of_trials: i32,
    pub averaging_mode: i16,
    pub undo_run_count: i16,
    pub first_episode_in_run: i16,
    pub trigger_threshold: f32,
    pub trigger_source: i16,
    pub trigger_action: i16,
    pub trigger_polarity: i16,
    pub scope_output_interval: f32,
    pub episode_start_to_start: f32,
    pub run_start_to_start: f32,
    pub trial_start_to_start: f32,
    pub average_count: i32,
    pub clock_change: i32,
    pub auto_trigger_strategy: i16,
}

#[derive(Debug)]
pub struct Group4DisplayParameters {
    pub drawing_strategy: i16,
    pub tiled_display: i16,
    pub erase_strategy: i16,
    pub data_display_mode: i16,
    pub display_average_update: i32,
    pub channel_stats_strategy: i16,
    pub calculation_period: i32,
    pub samples_per_trace: i32,
    pub start_display_num: i32,
    pub finish_display_num: i32,
    pub multi_color: i16,
    pub show_pn_raw_data: i16,
    pub statistics_period: f32,
    pub statistics_measurements: i32,
    pub statistics_save_strategy: i16,
}

#[derive(Debug)]
pub struct Group5HardwareInformation {
    pub adc_range: f32,
    pub dac_range: f32,
    pub adc_resolution: i32,
    pub dac_resolution: i32,
}

#[derive(Debug)]
pub struct Group6EnvironmentalInformation {
    pub experiment_type: i16,
    pub autosample_enable_legacy: i16,
    pub autosample_adc_num_legacy: i16,
    pub autosample_instrument_legacy: i16,
    pub autosample_addit_gain_legacy: f32,
    pub autosample_filter_legacy: f32,
    pub autosample_membrane_cap_legacy: f32,
    pub manual_info_strategy: i16,
    pub cell_id1: f32,
    pub cell_id2: f32,
    pub cell_id3: f32,
    pub creator_info: String,
    pub file_comment_legacy: String,
    pub file_start_millisecs: i16,
    pub comments_enable: i16,
    pub unused003a: [u8; 8],
}

#[derive(Debug)]
pub struct Group7MultiChannelInformation {
    pub adc_physical_to_logical_channel_map: [i16; 16],
    pub adc_sampling_seq: [i16; 16],
    pub adc_channel_name: [String; 16],
    pub adc_units: [String; 16],
    pub adc_programmable_gain: [f32; 16],
    pub adc_display_amplification: [f32; 16],
    pub adc_display_offset: [f32; 16],
    pub instrument_scale_factor: [f32; 16],
    pub instrument_offset: [f32; 16],
    pub signal_gain: [f32; 16],
    pub signal_offset: [f32; 16],
    pub signal_lowpass_filter: [f32; 16],
    pub signal_highpass_filter: [f32; 16],
    pub dac_channel_name: [String; 4],
    pub dac_channel_units: [String; 4],
    pub dac_scale_factor: [f32; 4],
    pub dac_holding_level: [f32; 4],
    pub signal_type: i16,
    pub unused004: [u8; 10],
}

#[derive(Debug)]
pub struct Group8SynchronousTimerOutputs {
    pub out_enable: i16,
    pub sample_number_out1: i16,
    pub sample_number_out2: i16,
    pub first_episode_out: i16,
    pub last_episode_out: i16,
    pub pulse_samples_out1: i16,
    pub pulse_samples_out2: i16,
}

#[derive(Debug)]
pub struct Group9EpochWaveformAndPulses {
    pub digital_enable: i16,
    pub waveform_source_legacy: i16,
    pub active_dac_channel: i16,
    pub inter_episode_level_legacy: i16,
    pub epoch_type_legacy: [i16; 10],
    pub epoch_init_level_legacy: [f32; 10],
    pub epoch_level_inc_legacy: [f32; 10],
    pub epoch_init_duration_legacy: [i16; 10],
    pub epoch_duration_inc_legacy: [i16; 10],
    pub digital_holding: i16,
    pub digital_inter_episode: i16,
    pub digital_value: [i16; 10],
    pub unavailable1608: [u8; 4],
    pub digital_dac_channel: i16,
    pub unused005: [u8; 6],
}

#[derive(Debug)]
pub struct Group10DacOutputFile {
    pub dac_file_scale_legacy: f32,
    pub dac_file_offset_legacy: f32,
    pub unused006: [u8; 2],
    pub dac_file_episode_num_legacy: i16,
    pub dac_file_adc_num_legacy: i16,
    pub dac_file_path_legacy: String,
}

#[derive(Debug)]
pub struct Group11PresweepConditioningPulseTrain {
    pub condit_enable_legacy: i16,
    pub condit_channel_legacy: i16,
    pub condit_num_pulses_legacy: i32,
    pub baseline_duration_legacy: f32,
    pub baseline_level_legacy: f32,
    pub step_duration_legacy: f32,
    pub step_level_legacy: f32,
    pub post_train_period_legacy: f32,
    pub post_train_level_legacy: f32,
    pub unused007: [u8; 12],
}

#[derive(Debug)]
pub struct Group12VariableParameterUserList {
    pub param_to_vary_legacy: i16,
    pub param_value_list_legacy: String,
}

#[derive(Debug)]
pub struct Group13AutopeakMeasurement {
    pub autopeak_enable_legacy: i16,
    pub autopeak_polarity_legacy: i16,
    pub autopeak_adc_num_legacy: i16,
    pub autopeak_search_mode_legacy: i16,
    pub autopeak_start_legacy: i32,
    pub autopeak_end_legacy: i32,
    pub autopeak_smoothing_legacy: i16,
    pub autopeak_baseline_legacy: i16,
    pub autopeak_average_legacy: i16,
    pub unavailable1866: [u8; 2],
    pub autopeak_baseline_start_legacy: i32,
    pub autopeak_baseline_end_legacy: i32,
    pub autopeak_measurements_legacy: i32,
}

#[derive(Debug)]
pub struct Group14ChannelArithmetic {
    pub arithmetic_enable: i16,
    pub arithmetic_upper_limit: f32,
    pub arithmetic_lower_limit: f32,
    pub arithmetic_adc_num_a: i16,
    pub arithmetic_adc_num_b: i16,
    pub arithmetic_k1: f32,
    pub arithmetic_k2: f32,
    pub arithmetic_k3: f32,
    pub arithmetic_k4: f32,
    pub arithmetic_operator: String,
    pub arithmetic_units: String,
    pub arithmetic_k5: f32,
    pub arithmetic_k6: f32,
    pub arithmetic_expression: i16,
    pub unused008: [u8; 2],
}

#[derive(Debug)]
pub struct Group15OnlineSubtraction {
    pub pn_enable_legacy: i16,
    pub pn_position: i16,
    pub pn_polarity_legacy: i16,
    pub pn_num_pulses: i16,
    pub pn_adc_num_legacy: i16,
    pub pn_holding_level_legacy: f32,
    pub pn_settling_time: f32,
    pub pn_interpulse: f32,
    pub unused009: [u8; 12],
}

#[derive(Debug)]
pub struct Group16MiscParameters {
    pub list_enable_legacy: i16,
    pub bell_enable: [i16; 2],
    pub bell_location: [i16; 2],
    pub bell_repetitions: [i16; 2],
    pub level_hysteresis: i16,
    pub time_hysteresis: i32,
    pub allow_external_tags: i16,
    pub lowpass_filter_type: [u8; 16],
    pub highpass_filter_type: [u8; 16],
    pub average_algorithm: i16,
    pub average_weighting: f32,
    pub undo_prompt_strategy: i16,
    pub trial_trigger_source: i16,
    pub statistics_display_strategy: i16,
    pub external_tag_type: i16,
    pub header_size: i32,
    pub file_duration_legacy: f64,
    pub statistics_display_strategy_dup: i16,
}

#[derive(Debug)]
pub struct ExtGroup2FileStructure {
    pub dac_file_ptr: [i32; 2],
    pub dac_file_num_episodes: [i32; 2],
}

#[derive(Debug)]
pub struct ExtGroup3TrialHierarchy {
    pub first_run_delay: f32,
    pub unused010: [u8; 6],
}

#[derive(Debug)]
pub struct ExtGroup7MultiChannel {
    pub dac_calibration_factor: [f32; 4],
    pub dac_calibration_offset: [f32; 4],
    pub unused011: [u8; 30],
}

#[derive(Debug)]
pub struct Group17TrainParameters {
    pub epoch_pulse_period: [i32; 20],
    pub epoch_pulse_width: [i32; 20],
}

#[derive(Debug)]
pub struct ExtGroup9Epoch {
    pub waveform_enable: [i16; 2],
    pub waveform_source: [i16; 2],
    pub inter_episode_level: [i16; 2],
    pub epoch_type: [i16; 20],
    pub epoch_init_level: [f32; 20],
    pub epoch_level_inc: [f32; 20],
    pub epoch_init_duration: [i32; 20],
    pub epoch_duration_inc: [i32; 20],
    pub digital_train_value: [i16; 10],
    pub digital_train_active_logic: i16,
    pub unused012: [u8; 18],
}

#[derive(Debug)]
pub struct ExtGroup10DacOutputFile {
    pub dac_file_scale: [f32; 2],
    pub dac_file_offset: [f32; 2],
    pub dac_file_episode_num: [i32; 2],
    pub dac_file_adc_num: [i16; 2],
    pub dac_file_path: [String; 2],
    pub unused013: [u8; 12],
}

#[derive(Debug)]
pub struct ExtGroup11Presweep {
    pub condit_enable: [i16; 2],
    pub condit_num_pulses: [i32; 2],
    pub baseline_duration: [f32; 2],
    pub baseline_level: [f32; 2],
    pub step_duration: [f32; 2],
    pub step_level: [f32; 2],
    pub post_train_period: [f32; 2],
    pub post_train_level: [f32; 2],
    pub unused014: [u8; 40],
}

#[derive(Debug)]
pub struct ExtGroup12VariableParameter {
    pub ul_enable: [i16; 4],
    pub ul_param_to_vary: [i16; 4],
    pub ul_param_value_list: [String; 4],
    pub ul_repeat: [i16; 4],
    pub unused015: [u8; 48],
}

#[derive(Debug)]
pub struct ExtGroup6Environmental {
    pub telegraph_enable: [i16; 16],
    pub telegraph_instrument: [i16; 16],
    pub telegraph_addit_gain: [f32; 16],
    pub telegraph_filter: [f32; 16],
    pub telegraph_membrane_cap: [f32; 16],
    pub telegraph_mode: [i16; 16],
    pub telegraph_dac_scale_factor_enable: [i16; 4],
    pub unused016a: [u8; 24],
    pub auto_analyse_enable: i16,
    pub auto_analysis_macro_name: String,
    pub protocol_path: String,
    pub file_comment: String,
    pub file_guid: [u8; 16],
    pub instrument_holding_level: [f32; 4],
    pub file_crc: i32,
    pub modifier_info: String,
    pub unused17: [u8; 76],
}

#[derive(Debug)]
pub struct Group13StatisticsMeasurements {
    pub stats_enable: i16,
    pub stats_active_channels: u16,
    pub stats_search_region_flags: u16,
    pub stats_selected_region: i16,
    pub stats_search_mode_legacy: i16,
    pub stats_smoothing: i16,
    pub stats_smoothing_enable: i16,
    pub stats_baseline: i16,
    pub stats_baseline_start: i32,
    pub stats_baseline_end: i32,
    pub stats_measurements: [i32; 8],
    pub stats_start: [i32; 8],
    pub stats_end: [i32; 8],
    pub rise_bottom_percentile: [i16; 8],
    pub rise_top_percentile: [i16; 8],
    pub decay_bottom_percentile: [i16; 8],
    pub decay_top_percentile: [i16; 8],
    pub stats_channel_polarity: [i16; 16],
    pub stats_search_mode: [i16; 8],
    pub unused018: [u8; 156],
}

#[derive(Debug)]
pub struct Group18ApplicationVersionData {
    pub major_version: i16,
    pub minor_version: i16,
    pub bugfix_version: i16,
    pub build_version: i16,
    pub modifier_major_version: i16,
    pub modifier_minor_version: i16,
    pub modifier_bugfix_version: i16,
    pub modifier_build_version: i16,
}

#[derive(Debug)]
pub struct Group19LtpProtocol {
    pub ltp_type: i16,
    pub ltp_usage_of_dac: [i16; 2],
    pub ltp_presynaptic_pulses: [i16; 2],
    pub unused020: [u8; 4],
}

#[derive(Debug)]
pub struct Group20Dd132xTriggerOut {
    pub dd132x_trigger_out: i16,
    pub unused021: [u8; 6],
}

#[derive(Debug)]
pub struct Group21EpochResistance {
    pub epoch_resistance_signal_name: [String; 2],
    pub epoch_resistance_state: [i16; 2],
    pub unused022: [u8; 16],
}

#[derive(Debug)]
pub struct Group22AlternatingEpisodicMode {
    pub alternate_dac_output_state: i16,
    pub alternate_digital_value: [i16; 10],
    pub alternate_digital_train_value: [i16; 10],
    pub alternate_digital_output_state: i16,
    pub unused023: [u8; 14],
}

#[derive(Debug)]
pub struct Group23PostProcessingActions {
    pub post_process_lowpass_filter: [f32; 16],
    pub post_process_lowpass_filter_type: [u8; 16],
    pub unused2048: [u8; 130],
}

fn read_i16(buf: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap())
}

fn read_u16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap())
}

fn read_i32(buf: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap())
}

fn read_f32(buf: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap())
}

fn read_f64(buf: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap())
}

fn read_bytes<const N: usize>(buf: &[u8], offset: usize) -> [u8; N] {
    buf[offset..offset + N].try_into().unwrap()
}

fn read_string(buf: &[u8], offset: usize, len: usize) -> String {
    let raw = &buf[offset..offset + len];
    let text = String::from_utf8_lossy(raw);
    text.trim_end_matches('\0').trim_end().to_string()
}

fn read_i16_array<const N: usize>(buf: &[u8], offset: usize) -> [i16; N] {
    std::array::from_fn(|i| read_i16(buf, offset + i * 2))
}

fn read_i32_array<const N: usize>(buf: &[u8], offset: usize) -> [i32; N] {
    std::array::from_fn(|i| read_i32(buf, offset + i * 4))
}

fn read_f32_array<const N: usize>(buf: &[u8], offset: usize) -> [f32; N] {
    std::array::from_fn(|i| read_f32(buf, offset + i * 4))
}

fn read_string_array<const N: usize>(buf: &[u8], offset: usize, item_len: usize) -> [String; N] {
    std::array::from_fn(|i| read_string(buf, offset + i * item_len, item_len))
}

fn parse_abf_v1_header(buf: &[u8], options: &AbfHeaderReadOptions) -> AbfV1Header {
    let group1_file_id = if options.group1_file_id {
        Some(Group1FileIdAndSize {
            file_signature: read_string(buf, 0, 4),
            file_version_number: read_f32(buf, 4),
            operation_mode: read_i16(buf, 8),
            actual_acq_length: read_i32(buf, 10),
            num_points_ignored: read_i16(buf, 14),
            actual_episodes: read_i32(buf, 16),
            file_start_date: read_i32(buf, 20),
            file_start_time: read_i32(buf, 24),
            stopwatch_time: read_i32(buf, 28),
            header_version_number: read_f32(buf, 32),
            file_type: read_i16(buf, 36),
            ms_bin_format: read_i16(buf, 38),
        })
    } else {
        None
    };

    let group2_file_structure = if options.group2_file_structure {
        Some(Group2FileStructure {
            data_section_ptr: read_i32(buf, 40),
            tag_section_ptr: read_i32(buf, 44),
            num_tag_entries: read_i32(buf, 48),
            scope_config_ptr: read_i32(buf, 52),
            num_scopes: read_i32(buf, 56),
            dac_file_ptr_legacy: read_i32(buf, 60),
            dac_file_num_episodes_legacy: read_i32(buf, 64),
            unused68: read_bytes(buf, 68),
            delta_array_ptr: read_i32(buf, 72),
            num_deltas: read_i32(buf, 76),
            voice_tag_ptr: read_i32(buf, 80),
            voice_tag_entries: read_i32(buf, 84),
            unused88: read_i32(buf, 88),
            synch_array_ptr: read_i32(buf, 92),
            synch_array_size: read_i32(buf, 96),
            data_format: read_i16(buf, 100),
            simultaneous_scan: read_i16(buf, 102),
            statistics_config_ptr: read_i32(buf, 104),
            annotation_section_ptr: read_i32(buf, 108),
            num_annotations: read_i32(buf, 112),
            unused004: read_bytes(buf, 116),
        })
    } else {
        None
    };

    let group3_trial_hierarchy = if options.group3_trial_hierarchy {
        Some(Group3TrialHierarchy {
            channel_count_acquired: read_i16(buf, 118),
            adc_num_channels: read_i16(buf, 120),
            adc_sample_interval: read_f32(buf, 122),
            adc_second_sample_interval: read_f32(buf, 126),
            synch_time_unit: read_f32(buf, 130),
            seconds_per_run: read_f32(buf, 134),
            num_samples_per_episode: read_i32(buf, 138),
            pre_trigger_samples: read_i32(buf, 142),
            episodes_per_run: read_i32(buf, 146),
            runs_per_trial: read_i32(buf, 150),
            number_of_trials: read_i32(buf, 154),
            averaging_mode: read_i16(buf, 158),
            undo_run_count: read_i16(buf, 160),
            first_episode_in_run: read_i16(buf, 162),
            trigger_threshold: read_f32(buf, 164),
            trigger_source: read_i16(buf, 168),
            trigger_action: read_i16(buf, 170),
            trigger_polarity: read_i16(buf, 172),
            scope_output_interval: read_f32(buf, 174),
            episode_start_to_start: read_f32(buf, 178),
            run_start_to_start: read_f32(buf, 182),
            trial_start_to_start: read_f32(buf, 186),
            average_count: read_i32(buf, 190),
            clock_change: read_i32(buf, 194),
            auto_trigger_strategy: read_i16(buf, 198),
        })
    } else {
        None
    };

    let group4_display = if options.group4_display {
        Some(Group4DisplayParameters {
            drawing_strategy: read_i16(buf, 200),
            tiled_display: read_i16(buf, 202),
            erase_strategy: read_i16(buf, 204),
            data_display_mode: read_i16(buf, 206),
            display_average_update: read_i32(buf, 208),
            channel_stats_strategy: read_i16(buf, 212),
            calculation_period: read_i32(buf, 214),
            samples_per_trace: read_i32(buf, 218),
            start_display_num: read_i32(buf, 222),
            finish_display_num: read_i32(buf, 226),
            multi_color: read_i16(buf, 230),
            show_pn_raw_data: read_i16(buf, 232),
            statistics_period: read_f32(buf, 234),
            statistics_measurements: read_i32(buf, 238),
            statistics_save_strategy: read_i16(buf, 242),
        })
    } else {
        None
    };

    let group5_hardware = if options.group5_hardware {
        Some(Group5HardwareInformation {
            adc_range: read_f32(buf, 244),
            dac_range: read_f32(buf, 248),
            adc_resolution: read_i32(buf, 252),
            dac_resolution: read_i32(buf, 256),
        })
    } else {
        None
    };

    let group6_environment = if options.group6_environment {
        Some(Group6EnvironmentalInformation {
            experiment_type: read_i16(buf, 260),
            autosample_enable_legacy: read_i16(buf, 262),
            autosample_adc_num_legacy: read_i16(buf, 264),
            autosample_instrument_legacy: read_i16(buf, 266),
            autosample_addit_gain_legacy: read_f32(buf, 268),
            autosample_filter_legacy: read_f32(buf, 272),
            autosample_membrane_cap_legacy: read_f32(buf, 276),
            manual_info_strategy: read_i16(buf, 280),
            cell_id1: read_f32(buf, 282),
            cell_id2: read_f32(buf, 286),
            cell_id3: read_f32(buf, 290),
            creator_info: read_string(buf, 294, 16),
            file_comment_legacy: read_string(buf, 310, 56),
            file_start_millisecs: read_i16(buf, 366),
            comments_enable: read_i16(buf, 368),
            unused003a: read_bytes(buf, 370),
        })
    } else {
        None
    };

    let group7_multichannel = if options.group7_multichannel {
        Some(Group7MultiChannelInformation {
            adc_physical_to_logical_channel_map: read_i16_array(buf, 378),
            adc_sampling_seq: read_i16_array(buf, 410),
            adc_channel_name: read_string_array(buf, 442, 10),
            adc_units: read_string_array(buf, 602, 8),
            adc_programmable_gain: read_f32_array(buf, 730),
            adc_display_amplification: read_f32_array(buf, 794),
            adc_display_offset: read_f32_array(buf, 858),
            instrument_scale_factor: read_f32_array(buf, 922),
            instrument_offset: read_f32_array(buf, 986),
            signal_gain: read_f32_array(buf, 1050),
            signal_offset: read_f32_array(buf, 1114),
            signal_lowpass_filter: read_f32_array(buf, 1178),
            signal_highpass_filter: read_f32_array(buf, 1242),
            dac_channel_name: read_string_array(buf, 1306, 10),
            dac_channel_units: read_string_array(buf, 1346, 8),
            dac_scale_factor: read_f32_array(buf, 1378),
            dac_holding_level: read_f32_array(buf, 1394),
            signal_type: read_i16(buf, 1410),
            unused004: read_bytes(buf, 1412),
        })
    } else {
        None
    };

    let group8_sync_timer = if options.group8_sync_timer {
        Some(Group8SynchronousTimerOutputs {
            out_enable: read_i16(buf, 1422),
            sample_number_out1: read_i16(buf, 1424),
            sample_number_out2: read_i16(buf, 1426),
            first_episode_out: read_i16(buf, 1428),
            last_episode_out: read_i16(buf, 1430),
            pulse_samples_out1: read_i16(buf, 1432),
            pulse_samples_out2: read_i16(buf, 1434),
        })
    } else {
        None
    };

    let group9_epoch = if options.group9_epoch {
        Some(Group9EpochWaveformAndPulses {
            digital_enable: read_i16(buf, 1436),
            waveform_source_legacy: read_i16(buf, 1438),
            active_dac_channel: read_i16(buf, 1440),
            inter_episode_level_legacy: read_i16(buf, 1442),
            epoch_type_legacy: read_i16_array(buf, 1444),
            epoch_init_level_legacy: read_f32_array(buf, 1464),
            epoch_level_inc_legacy: read_f32_array(buf, 1504),
            epoch_init_duration_legacy: read_i16_array(buf, 1544),
            epoch_duration_inc_legacy: read_i16_array(buf, 1564),
            digital_holding: read_i16(buf, 1584),
            digital_inter_episode: read_i16(buf, 1586),
            digital_value: read_i16_array(buf, 1588),
            unavailable1608: read_bytes(buf, 1608),
            digital_dac_channel: read_i16(buf, 1612),
            unused005: read_bytes(buf, 1614),
        })
    } else {
        None
    };

    let group10_dac_output_file = if options.group10_dac_output_file {
        Some(Group10DacOutputFile {
            dac_file_scale_legacy: read_f32(buf, 1620),
            dac_file_offset_legacy: read_f32(buf, 1624),
            unused006: read_bytes(buf, 1628),
            dac_file_episode_num_legacy: read_i16(buf, 1630),
            dac_file_adc_num_legacy: read_i16(buf, 1632),
            dac_file_path_legacy: read_string(buf, 1634, 84),
        })
    } else {
        None
    };

    let group11_presweep = if options.group11_presweep {
        Some(Group11PresweepConditioningPulseTrain {
            condit_enable_legacy: read_i16(buf, 1718),
            condit_channel_legacy: read_i16(buf, 1720),
            condit_num_pulses_legacy: read_i32(buf, 1722),
            baseline_duration_legacy: read_f32(buf, 1726),
            baseline_level_legacy: read_f32(buf, 1730),
            step_duration_legacy: read_f32(buf, 1734),
            step_level_legacy: read_f32(buf, 1738),
            post_train_period_legacy: read_f32(buf, 1742),
            post_train_level_legacy: read_f32(buf, 1746),
            unused007: read_bytes(buf, 1750),
        })
    } else {
        None
    };

    let group12_variable_parameter = if options.group12_variable_parameter {
        Some(Group12VariableParameterUserList {
            param_to_vary_legacy: read_i16(buf, 1762),
            param_value_list_legacy: read_string(buf, 1764, 80),
        })
    } else {
        None
    };

    let group13_autopeak = if options.group13_autopeak {
        Some(Group13AutopeakMeasurement {
            autopeak_enable_legacy: read_i16(buf, 1844),
            autopeak_polarity_legacy: read_i16(buf, 1846),
            autopeak_adc_num_legacy: read_i16(buf, 1848),
            autopeak_search_mode_legacy: read_i16(buf, 1850),
            autopeak_start_legacy: read_i32(buf, 1852),
            autopeak_end_legacy: read_i32(buf, 1856),
            autopeak_smoothing_legacy: read_i16(buf, 1860),
            autopeak_baseline_legacy: read_i16(buf, 1862),
            autopeak_average_legacy: read_i16(buf, 1864),
            unavailable1866: read_bytes(buf, 1866),
            autopeak_baseline_start_legacy: read_i32(buf, 1868),
            autopeak_baseline_end_legacy: read_i32(buf, 1872),
            autopeak_measurements_legacy: read_i32(buf, 1876),
        })
    } else {
        None
    };

    let group14_arithmetic = if options.group14_arithmetic {
        Some(Group14ChannelArithmetic {
            arithmetic_enable: read_i16(buf, 1880),
            arithmetic_upper_limit: read_f32(buf, 1882),
            arithmetic_lower_limit: read_f32(buf, 1886),
            arithmetic_adc_num_a: read_i16(buf, 1890),
            arithmetic_adc_num_b: read_i16(buf, 1892),
            arithmetic_k1: read_f32(buf, 1894),
            arithmetic_k2: read_f32(buf, 1898),
            arithmetic_k3: read_f32(buf, 1902),
            arithmetic_k4: read_f32(buf, 1906),
            arithmetic_operator: read_string(buf, 1910, 2),
            arithmetic_units: read_string(buf, 1912, 8),
            arithmetic_k5: read_f32(buf, 1920),
            arithmetic_k6: read_f32(buf, 1924),
            arithmetic_expression: read_i16(buf, 1928),
            unused008: read_bytes(buf, 1930),
        })
    } else {
        None
    };

    let group15_online_subtraction = if options.group15_online_subtraction {
        Some(Group15OnlineSubtraction {
            pn_enable_legacy: read_i16(buf, 1932),
            pn_position: read_i16(buf, 1934),
            pn_polarity_legacy: read_i16(buf, 1936),
            pn_num_pulses: read_i16(buf, 1938),
            pn_adc_num_legacy: read_i16(buf, 1940),
            pn_holding_level_legacy: read_f32(buf, 1942),
            pn_settling_time: read_f32(buf, 1946),
            pn_interpulse: read_f32(buf, 1950),
            unused009: read_bytes(buf, 1954),
        })
    } else {
        None
    };

    let group16_misc = if options.group16_misc {
        Some(Group16MiscParameters {
            list_enable_legacy: read_i16(buf, 1966),
            bell_enable: read_i16_array(buf, 1968),
            bell_location: read_i16_array(buf, 1972),
            bell_repetitions: read_i16_array(buf, 1976),
            level_hysteresis: read_i16(buf, 1980),
            time_hysteresis: read_i32(buf, 1982),
            allow_external_tags: read_i16(buf, 1986),
            lowpass_filter_type: read_bytes(buf, 1988),
            highpass_filter_type: read_bytes(buf, 2004),
            average_algorithm: read_i16(buf, 2020),
            average_weighting: read_f32(buf, 2022),
            undo_prompt_strategy: read_i16(buf, 2026),
            trial_trigger_source: read_i16(buf, 2028),
            statistics_display_strategy: read_i16(buf, 2030),
            external_tag_type: read_i16(buf, 2032),
            header_size: read_i32(buf, 2034),
            file_duration_legacy: read_f64(buf, 2038),
            statistics_display_strategy_dup: read_i16(buf, 2046),
        })
    } else {
        None
    };

    let ext_group2_file_structure = if options.ext_group2_file_structure {
        Some(ExtGroup2FileStructure {
            dac_file_ptr: read_i32_array(buf, 2048),
            dac_file_num_episodes: read_i32_array(buf, 2056),
        })
    } else {
        None
    };

    let ext_group3_trial_hierarchy = if options.ext_group3_trial_hierarchy {
        Some(ExtGroup3TrialHierarchy {
            first_run_delay: read_f32(buf, 2064),
            unused010: read_bytes(buf, 2068),
        })
    } else {
        None
    };

    let ext_group7_multichannel = if options.ext_group7_multichannel {
        Some(ExtGroup7MultiChannel {
            dac_calibration_factor: read_f32_array(buf, 2074),
            dac_calibration_offset: read_f32_array(buf, 2090),
            unused011: read_bytes(buf, 2106),
        })
    } else {
        None
    };

    let group17_train = if options.group17_train {
        Some(Group17TrainParameters {
            epoch_pulse_period: read_i32_array(buf, 2136),
            epoch_pulse_width: read_i32_array(buf, 2216),
        })
    } else {
        None
    };

    let ext_group9_epoch = if options.ext_group9_epoch {
        Some(ExtGroup9Epoch {
            waveform_enable: read_i16_array(buf, 2296),
            waveform_source: read_i16_array(buf, 2300),
            inter_episode_level: read_i16_array(buf, 2304),
            epoch_type: read_i16_array(buf, 2308),
            epoch_init_level: read_f32_array(buf, 2348),
            epoch_level_inc: read_f32_array(buf, 2428),
            epoch_init_duration: read_i32_array(buf, 2508),
            epoch_duration_inc: read_i32_array(buf, 2588),
            digital_train_value: read_i16_array(buf, 2668),
            digital_train_active_logic: read_i16(buf, 2688),
            unused012: read_bytes(buf, 2690),
        })
    } else {
        None
    };

    let ext_group10_dac_output_file = if options.ext_group10_dac_output_file {
        Some(ExtGroup10DacOutputFile {
            dac_file_scale: read_f32_array(buf, 2708),
            dac_file_offset: read_f32_array(buf, 2716),
            dac_file_episode_num: read_i32_array(buf, 2724),
            dac_file_adc_num: read_i16_array(buf, 2732),
            dac_file_path: read_string_array(buf, 2736, 206),
            unused013: read_bytes(buf, 3248),
        })
    } else {
        None
    };

    let ext_group11_presweep = if options.ext_group11_presweep {
        Some(ExtGroup11Presweep {
            condit_enable: read_i16_array(buf, 3260),
            condit_num_pulses: read_i32_array(buf, 3264),
            baseline_duration: read_f32_array(buf, 3272),
            baseline_level: read_f32_array(buf, 3280),
            step_duration: read_f32_array(buf, 3288),
            step_level: read_f32_array(buf, 3296),
            post_train_period: read_f32_array(buf, 3304),
            post_train_level: read_f32_array(buf, 3312),
            unused014: read_bytes(buf, 3320),
        })
    } else {
        None
    };

    let ext_group12_variable_parameter = if options.ext_group12_variable_parameter {
        Some(ExtGroup12VariableParameter {
            ul_enable: read_i16_array(buf, 3360),
            ul_param_to_vary: read_i16_array(buf, 3368),
            ul_param_value_list: read_string_array(buf, 3376, 256),
            ul_repeat: read_i16_array(buf, 4400),
            unused015: read_bytes(buf, 4408),
        })
    } else {
        None
    };

    let ext_group6_environment = if options.ext_group6_environment {
        Some(ExtGroup6Environmental {
            telegraph_enable: read_i16_array(buf, 4512),
            telegraph_instrument: read_i16_array(buf, 4544),
            telegraph_addit_gain: read_f32_array(buf, 4576),
            telegraph_filter: read_f32_array(buf, 4640),
            telegraph_membrane_cap: read_f32_array(buf, 4704),
            telegraph_mode: read_i16_array(buf, 4768),
            telegraph_dac_scale_factor_enable: read_i16_array(buf, 4800),
            unused016a: read_bytes(buf, 4808),
            auto_analyse_enable: read_i16(buf, 4832),
            auto_analysis_macro_name: read_string(buf, 4834, 64),
            protocol_path: read_string(buf, 4898, 256),
            file_comment: read_string(buf, 5154, 128),
            file_guid: read_bytes(buf, 5282),
            instrument_holding_level: read_f32_array(buf, 5298),
            file_crc: read_i32(buf, 5314),
            modifier_info: read_string(buf, 5318, 16),
            unused17: read_bytes(buf, 5334),
        })
    } else {
        None
    };

    let group13_statistics = if options.group13_statistics {
        Some(Group13StatisticsMeasurements {
            stats_enable: read_i16(buf, 5410),
            stats_active_channels: read_u16(buf, 5412),
            stats_search_region_flags: read_u16(buf, 5414),
            stats_selected_region: read_i16(buf, 5416),
            stats_search_mode_legacy: read_i16(buf, 5418),
            stats_smoothing: read_i16(buf, 5420),
            stats_smoothing_enable: read_i16(buf, 5422),
            stats_baseline: read_i16(buf, 5424),
            stats_baseline_start: read_i32(buf, 5426),
            stats_baseline_end: read_i32(buf, 5430),
            stats_measurements: read_i32_array(buf, 5434),
            stats_start: read_i32_array(buf, 5466),
            stats_end: read_i32_array(buf, 5498),
            rise_bottom_percentile: read_i16_array(buf, 5530),
            rise_top_percentile: read_i16_array(buf, 5546),
            decay_bottom_percentile: read_i16_array(buf, 5562),
            decay_top_percentile: read_i16_array(buf, 5578),
            stats_channel_polarity: read_i16_array(buf, 5594),
            stats_search_mode: read_i16_array(buf, 5626),
            unused018: read_bytes(buf, 5642),
        })
    } else {
        None
    };

    let group18_app_version = if options.group18_app_version {
        Some(Group18ApplicationVersionData {
            major_version: read_i16(buf, 5798),
            minor_version: read_i16(buf, 5800),
            bugfix_version: read_i16(buf, 5802),
            build_version: read_i16(buf, 5804),
            modifier_major_version: read_i16(buf, 5806),
            modifier_minor_version: read_i16(buf, 5808),
            modifier_bugfix_version: read_i16(buf, 5810),
            modifier_build_version: read_i16(buf, 5812),
        })
    } else {
        None
    };

    let group19_ltp = if options.group19_ltp {
        Some(Group19LtpProtocol {
            ltp_type: read_i16(buf, 5814),
            ltp_usage_of_dac: read_i16_array(buf, 5816),
            ltp_presynaptic_pulses: read_i16_array(buf, 5820),
            unused020: read_bytes(buf, 5824),
        })
    } else {
        None
    };

    let group20_dd132x_trigger_out = if options.group20_dd132x_trigger_out {
        Some(Group20Dd132xTriggerOut {
            dd132x_trigger_out: read_i16(buf, 5828),
            unused021: read_bytes(buf, 5830),
        })
    } else {
        None
    };

    let group21_epoch_resistance = if options.group21_epoch_resistance {
        Some(Group21EpochResistance {
            epoch_resistance_signal_name: read_string_array(buf, 5836, 10),
            epoch_resistance_state: read_i16_array(buf, 5856),
            unused022: read_bytes(buf, 5860),
        })
    } else {
        None
    };

    let group22_alternating_episodic = if options.group22_alternating_episodic {
        Some(Group22AlternatingEpisodicMode {
            alternate_dac_output_state: read_i16(buf, 5892),
            alternate_digital_value: read_i16_array(buf, 5894),
            alternate_digital_train_value: read_i16_array(buf, 5914),
            alternate_digital_output_state: read_i16(buf, 5934),
            unused023: read_bytes(buf, 5936),
        })
    } else {
        None
    };

    let group23_post_processing = if options.group23_post_processing && buf.len() >= 6160 {
        Some(Group23PostProcessingActions {
            post_process_lowpass_filter: read_f32_array(buf, 5950),
            post_process_lowpass_filter_type: read_bytes(buf, 6014),
            unused2048: read_bytes(buf, 6030),
        })
    } else {
        None
    };

    AbfV1Header {
        group1_file_id,
        group2_file_structure,
        group3_trial_hierarchy,
        group4_display,
        group5_hardware,
        group6_environment,
        group7_multichannel,
        group8_sync_timer,
        group9_epoch,
        group10_dac_output_file,
        group11_presweep,
        group12_variable_parameter,
        group13_autopeak,
        group14_arithmetic,
        group15_online_subtraction,
        group16_misc,
        ext_group2_file_structure,
        ext_group3_trial_hierarchy,
        ext_group7_multichannel,
        group17_train,
        ext_group9_epoch,
        ext_group10_dac_output_file,
        ext_group11_presweep,
        ext_group12_variable_parameter,
        ext_group6_environment,
        group13_statistics,
        group18_app_version,
        group19_ltp,
        group20_dd132x_trigger_out,
        group21_epoch_resistance,
        group22_alternating_episodic,
        group23_post_processing,
    }
}

pub struct AbfReader {
    file: File,
    pub header: AbfV1Header,
}

impl AbfReader {
    /// Opens the ABF file and parses only Group 1 and Group 2 and Group 3 by default.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::open_with_options(path, AbfHeaderReadOptions::default())
    }

    /// Opens the ABF file and parses any enabled groups.
    pub fn open_with_options<P: AsRef<Path>>(path: P, options: AbfHeaderReadOptions) -> io::Result<Self> {
        let mut file = File::open(path)?;

        // ABF1 header is fixed-size and group-based (commonly 6144 bytes)
        let mut header_buf = vec![0u8; 6144];
        file.read_exact(&mut header_buf)?;

        // Offset 0 (4 bytes): Signature
        if &header_buf[0..4] != b"ABF " {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid ABF v1 file. (If it's 'ABF2', it requires a v2 parser.)",
            ));
        }

        let header = parse_abf_v1_header(&header_buf, &options);

        Ok(Self { file, header })
    }

    /// Reads the raw ADC data into a standard f32 Vec using memory-mapped I/O
    /// Memory-mapped I/O eliminates the intermediate byte buffer, reducing peak RAM by ~1GB
    /// Optimizations: caches header values, uses zero-copy memory mapping, parallel conversion
    pub fn read_raw_data(&mut self) -> io::Result<Vec<f32>> {
        // Extract header values once to avoid repeated .as_ref().unwrap() calls
        let group1 = self
            .header
            .group1_file_id
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Group1 not initialized"))?;
        let group2 = self
            .header
            .group2_file_structure
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Group2 not initialized"))?;

        let data_offset = (group2.data_section_ptr as u64) * 512;
        let samples = group1.actual_acq_length as usize;
        let data_format = group2.data_format;

        // Memory-map the file: OS handles paging, no explicit buffering needed
        let mmap = unsafe { Mmap::map(&self.file)? };
        let byte_data = &mmap[data_offset as usize..];

        match data_format {
            0 => {
                // 16-bit integers: convert directly from memory-mapped bytes in parallel
                // No intermediate byte_data vec allocation—reads directly from OS page cache
                let data = (0..samples)
                    .into_par_iter()
                    .map(|i| {
                        let offset = i * 2;
                        let arr = [byte_data[offset], byte_data[offset + 1]];
                        i16::from_le_bytes(arr) as f32
                    })
                    .collect();

                Ok(data)
            }
            1 => {
                // 32-bit float: convert directly from memory-mapped bytes in parallel
                let data = (0..samples)
                    .into_par_iter()
                    .map(|i| {
                        let offset = i * 4;
                        let arr = [byte_data[offset], byte_data[offset + 1], byte_data[offset + 2], byte_data[offset + 3]];
                        f32::from_le_bytes(arr)
                    })
                    .collect();

                Ok(data)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown data format: {}", data_format),
            )),
        }
    }

    pub fn read_channels(&mut self) -> io::Result<Vec<Vec<f32>>> {
        // Sample rate (Hz) for each channel
        //let sample_rate = (1e6 / group3.adc_sample_interval) as i16 / num_channels as i16;
        //println!("Sample rate: {} Hz", sample_rate);

        // Number of input channels recorded
        let num_channels: usize = self.header.group3_trial_hierarchy.as_ref().unwrap().adc_num_channels as usize;
        //println!("Number of channels: {}", num_channels);

        // Total number of samples acquired across all channels
        let acq_length = self.header.group1_file_id.as_ref().unwrap().actual_acq_length as usize;
        //println!("Total samples acquired: {}", acq_length);

        // Number of samples acquired per channel
        let per_channel_samples: usize = acq_length / num_channels;
        //println!("Samples per channel: {}", per_channel_samples);

        // Read the raw data and print the number of samples read
        let raw_data = self.read_raw_data().unwrap();
        //println!("Read {} samples of raw data", raw_data.len());

        // Split the raw data into separate channels
        let mut channels = vec![Vec::with_capacity(per_channel_samples); num_channels];
        for (i, &sample) in raw_data.iter().enumerate() {
            channels[i % num_channels].push(sample);
        }
        //println!("Split data into {} channels", channels.len());

        Ok(channels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abf_reader() {
        let mut reader = AbfReader::open_with_options(
            "../Example.abf",
            AbfHeaderReadOptions {
                group3_trial_hierarchy: true,
                ..Default::default()
            },
        )
        .unwrap();

        let channels = reader.read_channels().unwrap();

        println!("First 10 samples of channel 1: {:?}", &channels[0][..10]);
        println!("First 10 samples of channel 2: {:?}", &channels[1][..10]);
        println!("First 10 samples of channel 3: {:?}", &channels[2][..10]);
        println!("First 10 samples of channel 4: {:?}", &channels[3][..10]);
        println!("First 10 samples of channel 5: {:?}", &channels[4][..10]);
        println!("First 10 samples of channel 6: {:?}", &channels[5][..10]);
        println!("First 10 samples of channel 7: {:?}", &channels[6][..10]);
        println!("First 10 samples of channel 8: {:?}", &channels[7][..10]);
        println!("First 10 samples of channel 9: {:?}", &channels[8][..10]);
    }
}
