#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// std
use std::fmt;
use std::sync::Mutex;

// imgui
use imgui::*;

pub enum MessageCategory
{
    Info = 0,
    Warning = 1,
    Error = 2
}

impl fmt::Display for MessageCategory 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        match *self
        {
            MessageCategory::Info    => write!(f, "Info"),
            MessageCategory::Warning => write!(f, "Warning"),
            MessageCategory::Error   => write!(f, "Error"),
        }
    }
}

pub struct Message
{
    pub Text:     String,
    pub Category: MessageCategory
}

pub struct Console
{
    m_MessageBuffer: Vec<Message>,
}

lazy_static::lazy_static! 
{ 
    static ref s_Instance: Mutex<Console> = Mutex::new(Console { 
        m_MessageBuffer: vec![],
    });
}

impl Console
{
    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LogInfo(message: &str)
    {
        Console::AddMessage(MessageCategory::Info, message);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LogWarning(message: &str)
    {
        Console::AddMessage(MessageCategory::Warning, message);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn LogError(message: &str)
    {
        Console::AddMessage(MessageCategory::Error, message);
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    pub fn OnImGuiRender(ui: &Ui)
    {
        if let Some(console) = Window::new(im_str!("Console"))
                                          .size([500.0, 400.0], imgui::Condition::FirstUseEver)
                                          .begin(&ui)
        {
            if ui.button(im_str!("Clear"))
            {
                s_Instance.lock().unwrap().m_MessageBuffer.clear();
            }

            ui.separator();

            if let Some(consoleText) = ChildWindow::new(imgui::Id::from("ConsoleText")).begin(&ui)
            {
                for message in s_Instance.lock().unwrap().m_MessageBuffer.iter()
                {
                    let mut color: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    
                    match message.Category
                    {
                        MessageCategory::Info =>    { color = [0.0, 1.0, 0.0, 1.0]; },    
                        MessageCategory::Warning => { color = [1.0, 1.0, 0.0, 1.0]; },    
                        MessageCategory::Error =>   { color = [1.0, 0.0, 0.0, 1.0]; }    
                    }
    
                    ui.text_colored(color, message.Text.as_str());
                }

                if ui.scroll_y() >= ui.scroll_max_y()
                {
                    ui.set_scroll_here_y();
                }

                consoleText.end();
            }

            console.end();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------------------------------
    fn AddMessage(category: MessageCategory, message: &str)
    {
        let formattedMessage = format!("[{}] {}: {}", chrono::Local::now().format("%H:%M:%S").to_string(), category, message);

        let message = Message {
            Text: String::from(formattedMessage),
            Category: category
        };

        s_Instance.lock().unwrap().m_MessageBuffer.push(message);
    }
}
