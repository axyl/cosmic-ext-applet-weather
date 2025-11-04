use std::time::Duration;

use chrono::{Local, Timelike};

use crate::{
    config::{flags, APP_ID, Flags, MOON_ICON, SUN_ICON, WeatherConfig},
    fl,
    weather::{get_location_forecast, ObservationData},
};

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<Weather>(flags())
}

struct Weather {
    core: cosmic::app::Core,
    popup: Option<cosmic::iced::window::Id>,
    config: WeatherConfig,
    config_handler: Option<cosmic::cosmic_config::Config>,
    observation: ObservationData,
    latitude: String,
    longitude: String,
    use_fahrenheit: bool,
}

impl Weather {
    fn update_weather_data(&mut self) -> cosmic::app::Task<Message> {
        cosmic::Task::perform(
            get_location_forecast(
                self.config.latitude.to_string(),
                self.config.longitude.to_string(),
            ),
            |result| match result {
                Ok(observation) => {
                    cosmic::action::Action::App(Message::UpdateObservation(observation))
                }
                Err(error) => {
                    tracing::error!("Failed to get location forecast: {error:?}");
                    cosmic::action::Action::App(Message::UpdateObservation(
                        ObservationData::default(),
                    ))
                }
            },
        )
    }

    fn format_wind_details(&self) -> String {
        let direction = if self.observation.wind_dir.trim().is_empty() {
            "-"
        } else {
            self.observation.wind_dir.as_str()
        };

        match (self.observation.wind_spd_kt, self.observation.gust_kt) {
            (Some(speed), Some(gust)) => format!("{direction} {speed}kt gust {gust}kt"),
            (Some(speed), None) => format!("{direction} {speed}kt"),
            (None, Some(gust)) => format!("{direction} gust {gust}kt"),
            (None, None) => direction.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleWindow,
    PopupClosed(cosmic::iced::window::Id),
    UpdateObservation(ObservationData),
    UpdateLatitude(String),
    UpdateLongitude(String),
    ToggleFahrenheit(bool),
}

impl cosmic::Application for Weather {
    type Flags = Flags;
    type Message = Message;
    type Executor = cosmic::SingleThreadExecutor;

    const APP_ID: &'static str = APP_ID;

    fn init(
        core: cosmic::app::Core,
        flags: Self::Flags,
    ) -> (Self, cosmic::app::Task<Self::Message>) {
        let latitude = flags.config.latitude;
        let longitude = flags.config.longitude;
        let use_fahrenheit = flags.config.use_fahrenheit;

        (
            Self {
                core,
                popup: None,
                config: flags.config,
                config_handler: flags.config_handler,
                observation: ObservationData::default(),
                latitude: latitude.to_string(),
                longitude: longitude.to_string(),
                use_fahrenheit,
            },
            cosmic::task::message(Message::Tick),
        )
    }

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        cosmic::iced::time::every(Duration::from_secs(60)).map(|_| Message::Tick)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: cosmic::iced::window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::UpdateObservation(value) => {
                self.observation = value;
            }
            Message::Tick => {
                return self.update_weather_data();
            }
            Message::ToggleWindow => {
                if let Some(id) = self.popup.take() {
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(
                        id,
                    );
                } else {
                    let new_id = cosmic::iced::window::Id::unique();
                    self.popup.replace(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    return cosmic::iced::platform_specific::shell::commands::popup::get_popup(
                        popup_settings,
                    );
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::UpdateLatitude(value) => {
                self.latitude = value.to_string();

                if let Some(handler) = &self.config_handler
                    && let Err(error) = self
                        .config
                        .set_latitude(handler, value.parse::<f64>().unwrap_or_default())
                {
                    tracing::error!("{error}")
                }

                return self.update_weather_data();
            }
            Message::UpdateLongitude(value) => {
                self.longitude = value.to_string();

                if let Some(handler) = &self.config_handler
                    && let Err(error) = self
                        .config
                        .set_longitude(handler, value.parse::<f64>().unwrap_or_default())
                {
                    tracing::error!("{error}")
                }

                return self.update_weather_data();
            }
            Message::ToggleFahrenheit(value) => {
                self.use_fahrenheit = value;

                if let Some(handler) = &self.config_handler
                    && let Err(error) = self.config.set_use_fahrenheit(handler, value)
                {
                    tracing::error!("{error}")
                }
            }
        };

        cosmic::Task::none()
    }

    fn view(&self) -> cosmic::Element<'_, Message> {
        let icon_name = match Local::now().hour() {
            6..18 => SUN_ICON,
            _ => MOON_ICON,
        };

        let icon = cosmic::iced_widget::row![
            cosmic::widget::icon::from_name(icon_name)
                .size(14)
                .symbolic(true),
        ]
        .padding([3, 0, 0, 0]);
        let wind_details =
            cosmic::iced_widget::row![cosmic::iced_widget::text(self.format_wind_details())];

        let data =
            cosmic::Element::from(cosmic::iced_widget::row![icon, wind_details].spacing(4));
        let button = cosmic::widget::button::custom(data)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_down(Message::ToggleWindow);

        cosmic::widget::autosize::autosize(button, cosmic::widget::Id::unique()).into()
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> cosmic::Element<'_, Message> {
        let latitude_row = cosmic::iced_widget::column![
            cosmic::widget::text(fl!("latitude")),
            cosmic::widget::text_input(fl!("latitude"), &self.latitude)
                .on_input(Message::UpdateLatitude)
                .width(cosmic::iced::Length::Fill)
        ]
        .spacing(4);
        let longitude_row = cosmic::iced_widget::column![
            cosmic::widget::text(fl!("longitude")),
            cosmic::widget::text_input(fl!("longitude"), &self.longitude)
                .on_input(Message::UpdateLongitude)
                .width(cosmic::iced::Length::Fill)
        ]
        .spacing(4);
        let fahrenheit_toggler = cosmic::iced_widget::row![
            cosmic::widget::text(fl!("fahrenheit-toggle")),
            cosmic::widget::Space::with_width(cosmic::iced::Length::Fill),
            cosmic::widget::toggler(self.use_fahrenheit).on_toggle(Message::ToggleFahrenheit),
        ];

        let data = cosmic::iced_widget::column![
            cosmic::applet::padded_control(latitude_row),
            cosmic::applet::padded_control(longitude_row),
            cosmic::applet::padded_control(cosmic::widget::divider::horizontal::default()),
            cosmic::applet::padded_control(fahrenheit_toggler)
        ]
        .padding([16, 0]);

        self.core
            .applet
            .popup_container(cosmic::widget::container(data))
            .into()
    }
}
