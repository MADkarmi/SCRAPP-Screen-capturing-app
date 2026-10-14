//disable console window on windows
#![windows_subsystem = "windows"]

use std::println;
#[allow(unused_imports)]
use std::{
    path::{PathBuf},
    thread,
    env,
    sync::{
        Arc,
        Mutex
    }
};

use native_dialog::{FileDialog};//, MessageDialog, MessageType};

use slint::{
    invoke_from_event_loop,
    Model,
    ModelRc,
    SharedString,
    VecModel
};


slint::include_modules!();

//const SAMPLE_RATE: u32 = 16000;

use crate::{recorder_controller::ControlCommand};
mod recorder_controller;
use recorder_controller::RecorderController;

mod oauth;
mod upload_controller;
use upload_controller::{
    UploadManager,
    UploadMethod,
    UploadClient,
    UploadController,
};

mod appsettings;
mod mapuiobjects;
use mapuiobjects::{map_recording_file_format, map_recording_fps, map_recording_resolution};
use oauth::{
    OAuthConfig,
    session::OAuthSession,
};


fn main() -> Result<(), slint::PlatformError> {
    dotenvy::dotenv().ok();
    let oauth_session = OAuthSession::new();
    let oauth_config = OAuthConfig::from_env()
        .map_err(|e| format!("Failed to load OAuth configuration from environment variables: {e}"))?;

    let upload_controller : Arc<Mutex<Option<UploadController>>> = Arc::new(Mutex::new(None));

    let application_window = AppWindow::new()?;

    set_application_window_model_data(application_window.as_weak().unwrap());

    //add error handling
    let recorder_controller = RecorderController::new();


    /* start recording */
    // Handling the toggle recording state event from the UI. When the user clicks the "Start/Stop Recording" button, this closure is invoked.
    let controller = recorder_controller.clone();
    let recorder = application_window.as_weak();
    application_window.on_toggle_recording_state(move || {
        let recorder_handle = recorder.unwrap();
        let recording_state = !(recorder_handle.get_recordingState()); //change recording state
        let folder_path = recorder_handle.get_folderPath();
        recorder_handle.set_recordingState(recording_state);

        // Send the appropriate command to the recorder controller based on the new recording state.
        if recording_state{
            controller.send(ControlCommand::StartRecording {
                audio_sources: mapuiobjects::get_recording_targets(recorder_handle.get_audioSourceListModel()),
                video_sources: mapuiobjects::get_recording_targets(recorder_handle.get_videoSourceListModel()),
                output_path: PathBuf::from(folder_path.as_str()),
                fps: map_recording_fps(recorder_handle.get_fps().as_str()),
                resolution: map_recording_resolution(recorder_handle.get_resolution().as_str()),
                recording_format: map_recording_file_format(recorder_handle.get_fileFormat().as_str()),
            });

        }else {
            #[cfg(feature = "debug-info")]
            println!("Sent stop recording command");
            controller.send(ControlCommand::StopRecording);
        }

    });

    // Set up the source selection callback to handle user interactions with the source picker UI.
    // The callback will update the active state of the selected source and send an update command to the recorder controller.
    let controller = recorder_controller.clone();
    let source_picker_handle = application_window.as_weak();
    application_window.on_on_source_selected(move |selected_source, selection_mode|{
        let source_picker_handle = source_picker_handle.unwrap();
        let recorder_is_in_session = source_picker_handle.get_recordingState();
        let mut video_sources_updated = false;
        match selection_mode{
            SelectionMode::Single =>{
                let video_source_list_model = source_picker_handle.get_videoSourceListModel();
                video_source_list_model.iter().enumerate().for_each(|(index, source)|{
                    let mut data = source.clone();
                    if source.deviceId == selected_source.deviceId {
                        if !(data.isActive && recorder_is_in_session){
                            data.isActive = !data.isActive;
                            video_sources_updated = true;
                        }
                    }
                    else{
                        data.isActive = false;
                    }
                    video_source_list_model.set_row_data(index, data);
                });
                if video_source_list_model.iter().find(|source|{source.isActive}).is_some(){
                    source_picker_handle.set_enableRecording(true);
                } else {
                    source_picker_handle.set_enableRecording(false);
                };
                if video_sources_updated{
                    controller.send(ControlCommand::UpdateVideoSources {
                        video_sources: mapuiobjects::get_recording_targets(video_source_list_model.clone()),
                    });
                }
                source_picker_handle.set_videoSourceListModel(video_source_list_model);
            },
            SelectionMode::Multiple =>{
                let audio_source_list_model = source_picker_handle.get_audioSourceListModel();
                audio_source_list_model.iter().enumerate().for_each(|(index, source)|{
                    let mut data = source.clone();
                    if source.deviceId == selected_source.deviceId{
                        data.isActive = !data.isActive;
                    } else if source.deviceType == selected_source.deviceType{
                        data.isActive = false;
                    }
                    audio_source_list_model.set_row_data(index, data);
                });

                source_picker_handle.set_audioSourceListModel(audio_source_list_model);
            },
        }
    });

    // Set up the streaming state toggle callback to handle user interactions with the streaming state button in the UI.
    let streaming_state_btn_handle = application_window.as_weak();
    application_window.on_toggle_streaming_state(move || {
        let streaming_state_btn_handle = streaming_state_btn_handle.unwrap();
        streaming_state_btn_handle.set_streamingState(!(streaming_state_btn_handle.get_streamingState()));
    });

    // handle application closure event to hide the window instead of closing it, allowing for background operation.
    let application_closure_btn_handle = application_window.as_weak();
    application_window.on_request_application_closure(move || {
        let application_closure_btn_handle = application_closure_btn_handle.unwrap();
        application_closure_btn_handle.window().hide().unwrap();
    });

    // handling settings folder selection event from settings page the UI.
    let native_dialog_folder_pick_handle = application_window.as_weak();
    application_window.on_native_folder_picker(move || {
        let native_dialog_folder_pick_handle = native_dialog_folder_pick_handle.unwrap();
        let folder_path = FileDialog::new()
                                                    .set_location("~")
                                                    .show_open_single_dir().unwrap();

        let mut folder_path = match folder_path{
            None => PathBuf::new(),
            Some(path) => path,
        };

        native_dialog_folder_pick_handle.set_folderPath(
            SharedString::from(folder_path.as_mut_os_str().to_str().unwrap())
        );
    });

    // handle settings save event from settings page the UI.
    let save_settings_handle = application_window.as_weak();
    application_window.on_save_settings_to_file(move ||{
        let save_settings_handle = save_settings_handle.unwrap();

        let folder_path = save_settings_handle.get_folderPath();
        let file_format = save_settings_handle.get_fileFormat();
        let fps = save_settings_handle.get_fps();
        let resolution = save_settings_handle.get_resolution();

        match appsettings::write_to_file(
            folder_path.as_str(),
            file_format.as_str(),
            fps.as_str(),
            resolution.as_str(),
        ){
            Ok(_) => println!("Settings saved to file successfully."),
            Err(e) => println!("Error saving settings to file: {}", e),
        }
    });

    // handling login when user clicks the "Upload" button in the UI.
    // This closure initiates the OAuth login process and sets up the upload controller upon successful authentication.
    let login_session = oauth_session.clone();
    let login_config = oauth_config.clone();
    let switch_to_uploads_page_handle = application_window.as_weak();
    let upload_controller_initialization = upload_controller.clone();
    application_window.on_switch_to_uploads_page(move||{
        if login_session.is_authenticated(){
            let ui = switch_to_uploads_page_handle.unwrap();
            ui.set_switchToUploadsPage(true);
            return;
        }

        if let Some(ui) = switch_to_uploads_page_handle.upgrade() {
            ui.set_authenticating(true);
        }

        let switch_to_uploads_page_handle = switch_to_uploads_page_handle.clone();

        let session = login_session.clone();
        let config = login_config.clone();
        let upload_controller = upload_controller_initialization.clone();
        std::thread::spawn(move || {
            match oauth::login_user(&config){
                Ok(tokens) => {
                    #[cfg(feature = "oauth-session-debug-info")]
                    println!("$accessToken = {:#?}", tokens.access_token);

                    session.set(tokens);

                    //initialize upload manager and controller after successful login
                    let upload_client = UploadClient::new(
                        &env::var("UPLOAD_API_ENDPOINT")
                            .expect("UPLOAD_API_ENDPOINT must be set"),
                    );
                    let upload_manager = UploadManager::new(
                        config.clone(),
                        session.clone(),
                        upload_client,
                    );
                    let controller = UploadController::new(
                        Arc::new(upload_manager),
                    );

                    *upload_controller.lock().unwrap() = Some(controller);


                    #[cfg(feature = "oauth-session-debug-info")]
                    {
                        //getting user info from ID token
                        println!("tokens set in session: {:#?}", session.get());
                        println!("{:#?}", session.user());
                    }

                    let _ = invoke_from_event_loop(move || {
                        if let Some(ui) = switch_to_uploads_page_handle.upgrade() {
                            ui.set_authenticating(false);
                            ui.set_switchToUploadsPage(true);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("OAuth login failed: {e}");
                    let _ = invoke_from_event_loop(move || {
                    if let Some(ui) = switch_to_uploads_page_handle.upgrade() {
                        ui.set_authenticating(false);
                    }
                });
                }
            }
        });
    });

    // handling file selection event from upload page the UI.
    let native_dialog_file_pick_handle = application_window.as_weak();
    application_window.on_native_file_picker(move || {
        let native_dialog_file_pick_handle = native_dialog_file_pick_handle.unwrap();
        let file_path = FileDialog::new()
                                                    .set_location("~")
                                                    .add_filter("video", &["mp4"])
                                                    .show_open_single_file().unwrap();

        let mut file_path = match file_path{
            None => PathBuf::new(),
            Some(path) => path,
        };

        native_dialog_file_pick_handle.set_uploadFileName(
            SharedString::from(file_path.as_mut_os_str().to_str().unwrap())
        );
    });

    // handling upload events from the UI for different upload methods (stream, multipart, chunked).
    let ui_handle = application_window.as_weak();
    let upload_stream_controller = upload_controller.clone();
    application_window.on_upload_stream_video(move || {
        let Some(ui) = ui_handle.upgrade() else {
            eprintln!("Failed to upgrade UI handle");
            return;
        };

        let Some(controller) = upload_stream_controller.lock().unwrap().clone() else {
            eprintln!("Upload controller is not initialized");
            return;
        };

        let (file_path, title, description) = get_upload_data(&ui_handle.unwrap());

        controller.upload(
            file_path,
            title,
            description,
            UploadMethod::Stream,
            ui.as_weak()
        );
    });

    let ui_handle = application_window.as_weak();
    let upload_multipart_controller = upload_controller.clone();
    application_window.on_upload_multipart_video(move || {
        let Some(ui) = ui_handle.upgrade() else {
            eprintln!("Failed to upgrade UI handle");
            return;
        };

        let Some(controller) = upload_multipart_controller.lock().unwrap().clone() else {
            eprintln!("Upload controller is not initialized");
            return;
        };

        let (file_path, title, description) = get_upload_data(&ui_handle.unwrap());

        controller.upload(
            file_path,
            title,
            description,
            UploadMethod::Multipart,
            ui.as_weak()
        );
    });

    let ui_handle = application_window.as_weak();
    let upload_chunked_controller = upload_controller.clone();
    application_window.on_upload_chunked_video(move || {
        let Some(ui) = ui_handle.upgrade() else {
            eprintln!("Failed to upgrade UI handle");
            return;
        };

        let Some(controller) = upload_chunked_controller.lock().unwrap().clone() else {
            eprintln!("Upload controller is not initialized");
            return;
        };

        let (file_path, title, description) = get_upload_data(&ui_handle.unwrap());

        controller.upload(
            file_path,
            title,
            description,
            UploadMethod::Chunked,
            ui.as_weak()
        );
    });

    // handling logout event from the UI when user closes the upload.
    let logout_handle = application_window.as_weak();
    let logout_session = oauth_session.clone();
    let logout_config = oauth_config.clone();
    application_window.on_logout(move || {
        let session = logout_session.clone();
        let config = logout_config.clone();
        let logout_handle = logout_handle.clone();

        std::thread::spawn(move || {
            match oauth::logout_user(&session, &config) {
                Ok(()) => {
                    // let logout_handle = logout_handle.clone();
                    #[cfg(feature = "oauth-session-debug-info")]
                    println!("Logging out user...");
                    let _ = invoke_from_event_loop(move || {
                        if let Some(ui) = logout_handle.upgrade() {
                            ui.set_switchToUploadsPage(false);
                        }
                    });
                }

                Err(e) => {
                    eprintln!("Logout failed: {e}");
                }
            }
        });
    });




    application_window.run()
}

fn set_application_window_model_data(application_window: AppWindow){
    application_window.set_RecorderSupportedSettings(appsettings::load_supported_settings());

    let appsettings = appsettings::load_settings();
    application_window.set_folderPath(SharedString::from(appsettings.path));
    application_window.set_fileFormat(SharedString::from(appsettings.file_format));
    application_window.set_fps(SharedString::from(appsettings.fps));
    application_window.set_resolution(SharedString::from(appsettings.resolution));

    let (audio_sources, video_sources) = RecorderController::get_available_sources();
    application_window.set_audioSourceListModel(audio_sources);
    application_window.set_videoSourceListModel(video_sources);
}

fn get_upload_data(
    ui: &AppWindow,
) -> (String, String, String) {
    (
        ui.get_uploadFileName().to_string(),
        ui.get_uploadTitle().to_string(),
        ui.get_uploadDescription().to_string()
    )
}
