use relm4::gtk;
use gtk::CssProvider;
use gtk::{style_context_add_provider_for_display, style_context_remove_provider_for_display};
use gtk::gdk::Display;
use crate::Config;
use crate::css;
const CSS_TOP: &'static str = include_str!("./css/top.css");
const CSS_LOGO: &'static str = include_str!("./css/logo.css");
const CSS_NAVIGATION: &'static str = include_str!("./css/navigation.css");

const CSS_TOP_LIGHT: &'static str = include_str!("./css/top_light.css");
// const CSS_LOGO_LIGHT: &'static str = include_str!("./css/logo.css");
const CSS_NAVIGATION_LIGHT: &'static str = include_str!("./css/navigation_light.css");

const CSS_TOP_DARK: &'static str = include_str!("./css/top_dark.css");
// const CSS_LOGO_DARK: &'static str = include_str!("./css/logo.css");
const CSS_NAVIGATION_DARK: &'static str = include_str!("./css/navigation_dark.css");
pub struct Css{
    css_light: String,
    css_provider: CssProvider,
    css_provider_light: CssProvider,
    css_provider_dark: CssProvider
}
impl Css{
    pub fn new(config: &Config) -> Self{
        let css = String::from(CSS_TOP) + 
            CSS_LOGO + 
            CSS_NAVIGATION;
        let css_light = String::from(CSS_TOP_LIGHT) +
            CSS_LOGO +
            CSS_NAVIGATION_LIGHT;
        let css_dark = String::from(CSS_TOP_DARK) +
            CSS_LOGO +
            CSS_NAVIGATION_DARK;

        let css_provider = CssProvider::new();
        let css_provider_light = CssProvider::new();
        let css_provider_dark = CssProvider::new();

        css_provider.load_from_data(&css);
        css_provider_light.load_from_data(&css_light);
        css_provider_dark.load_from_data(&css_dark);
        style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
        Css { css_light, css_provider, css_provider_light, css_provider_dark}
    }
    pub fn set_normal(&self) {
        style_context_remove_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider_light
        );
        style_context_remove_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider_dark
        );
        style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }
    pub fn set_light(&self) {
        style_context_remove_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider
        );
        style_context_remove_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider_dark
        );
        style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider_light,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }
    pub fn set_dark(&self) {
        style_context_remove_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider
        );
        style_context_remove_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider_light
        );
        style_context_add_provider_for_display(
            &Display::default().unwrap(),
            &self.css_provider_dark,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }
}