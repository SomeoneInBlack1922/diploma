use std::sync::Arc;
use std::sync::RwLock;

use relm4::SimpleComponent;
use gtk::{Window, Box as GtkBox};
use relm4::component::Connector;
use relm4::prelude::*;
use relm4::gtk;
use gtk::prelude::*;
use crate::ui::navigation::NavigationView;
use crate::ui::navigation::NavigationOutput;
use crate::ui::work_area::settings::SettingsInit;
use crate::ui::work_area::settings::SettingsView;
use crate::bus::Bus;

pub struct TopWidgets {
    window: Window
}
enum Connectors{
    Empty,
    Setttings(Connector<SettingsView>)
}
pub struct TopView{
    bus: Arc<RwLock<Bus>>,
    top_container: GtkBox,
    working_area: GtkBox,
    navigation_controller: Controller<NavigationView>,
    connector: Connectors
}
#[derive(Debug)]
pub enum TopInput{
    Navigation(NavigationOutput)
}
impl SimpleComponent for TopView{
    type Input = TopInput;
    type Output = ();
    type Init = Bus;
    type Root = Window;
    type Widgets = ();
    fn init_root() -> Self::Root {
        Window::builder()
            .title("mainuscript")
            .height_request(800)
            .width_request(1500)
            .decorated(true)
            .vexpand(true)
            .maximized(true)
            .build()
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let bus = Arc::new(RwLock::new(init));
        //UI
        let top_container = GtkBox::new(gtk::Orientation::Horizontal, 0);
        let working_area = GtkBox::new(gtk::Orientation::Horizontal, 0);

        let empty_working_view = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .build();

        empty_working_view.add_css_class("working_area_empty");
        empty_working_view.append(&gtk::Label::new(Some("EMPTY WORKING AREA")));

        working_area.append(&empty_working_view);

        let navigation_bar_connector = NavigationView::builder()
            // .attach_to(&top_container)
            .launch(bus.clone());
        let navigation_bar_controller = navigation_bar_connector.forward(sender.input_sender(), |navigation_output|{
            return TopInput::Navigation(navigation_output)
        });
        top_container.append(navigation_bar_controller.widget());
        top_container.append(&working_area);

        root.set_child(Some(&top_container));

        ComponentParts {
            model: TopView {
                bus,
                top_container,
                // current_working_area: working_area_empty
                working_area,
                navigation_controller: navigation_bar_controller,
                connector:  Connectors::Empty
            },
            widgets: ()
        }
    }
    fn update(&mut self, message: Self::Input, _: ComponentSender<Self>) {
        match message{
            TopInput::Navigation(navigation_event) => {
                match navigation_event{
                    // Need to open settings in working area
                    NavigationOutput::Settings => {
                        tracing::info!("requested to open Settings");
                        // Remove previous working area
                        let previous_option = self.working_area.first_child();
                        if let Some(previous) = previous_option{
                            self.working_area.remove(&previous);
                        }
                        // Construct settings element
                        let setting_builder = SettingsView::builder();
                        let settings_root = &setting_builder.root;
                        
                        // Add settings wiew to the container
                        self.working_area.append(settings_root);
                        let setting_connector = setting_builder.launch(SettingsInit{
                            bus: self.bus.clone(),
                            highlighted_options: Arc::new(RwLock::new(0)),
                            init_control_message: "".into()
                        });
                        self.connector = Connectors::Setttings(setting_connector)
                    },
                    NavigationOutput::Object(_) => {
                        todo!()
                    }
                }
            }
        }
    }
    
}
fn new_top_container() -> GtkBox{
    GtkBox::new(gtk::Orientation::Horizontal, 0)
}