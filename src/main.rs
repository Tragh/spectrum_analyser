extern crate regex;
extern crate num;
#[macro_use] extern crate conrod;
mod support;
#[macro_use] extern crate glium;

extern crate find_folder;
extern crate rustfft;

use glium::{DisplayBuild, Surface};
use glium::backend::glutin_backend::GlutinFacade;

mod city2d;

pub mod waveformdrawer;
use waveformdrawer::{WaveformDrawer,WaveformDrawerSettings};

pub mod shared;
use shared::{AppState, WaveData, Ticker};

pub mod openbci_file;
use openbci_file::{OpenBCIFile};

pub mod glium_texdraw;
use glium_texdraw::{GliumTexDraw};



pub fn main() {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1000;

    println!("Building the window.");
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("Text Demo")
        .build_glium()
        .unwrap();

    println!("Constructing UI.");
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).theme(support::theme()).build();



    //OpenGL stuff###################################
    //let image = image::load(std::io::Cursor::new(&include_bytes!("../tests/fixture/opengl.png")[..]),image::PNG).unwrap().to_rgba();
    //let image_dimensions = image.dimensions();
    //let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    let gliumtexdraw=GliumTexDraw::new(&display);




    println!("Initialising internal data.");

    let mut app = AppState{
        show_file_widgets: true,
        file_selection: None,
        waveform_drawers: Vec::<WaveformDrawer>::new(),
        wave_data: None,
        ticker: Ticker::default()
    };


    // end waveform create

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let noto_sans = assets.join("fonts/NotoSans");


    // Specify the default font to use when none is specified by the widget.
    ui.theme.font_id = Some(ui.fonts.insert_from_file(noto_sans.join("NotoSans-Regular.ttf")).unwrap());

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    println!("Starting main event loop.");
    let mut frame_rater=support::FrameRater::new(0);
    let mut event_loop = support::EventLoop::new();
    //################################################################################################################################################################################################
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&display) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                ui.handle_event(event);
            }

            match event {
                // Break from the loop upon `Escape`.
                glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                glium::glutin::Event::Closed =>
                    break 'main,
                _ => {},
            }
        }



        let mut ticks;
        loop{
            ticks=app.ticker.ticks();
            for wfd in &mut app.waveform_drawers {wfd.update_stft(ticks, &app.wave_data);}
            frame_rater.fps(ticks);
            if frame_rater.elapsed_ms(ticks, 16) {break;}
        }




        let win_w : f64 = ui.win_w.clone();
        let win_h: f64 = ui.win_h.clone();
        let wx = |d: f64|->f64{d/win_w*2.0};
        let wy = |d: f64|->f64{d/win_h*2.0};


        set_ui(ui.set_widgets(), &ids, &display, &mut app);

        // Render the `Ui` and then display it on the screen.
        let mut redraw_ui: bool = false;
        let primitives = ui.draw_if_changed();
        let mut target = display.draw();
        if primitives.is_some() { redraw_ui=true; }
        if redraw_ui {renderer.fill(&display, primitives.unwrap(), &image_map);}
        if redraw_ui {
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
        }
            //###### MY DRAWING GOES HERE ######

            //gliumtexdraw.draw(&mut target,&textures[i as usize],0.0,wy(400.0-250.0*i as f64),wx(1600.0),wy(192.0));
            for wfd in &mut app.waveform_drawers {
                //gliumtexdraw.draw(&mut target,&waveform_textures[i as usize],0.0,wy(400.0-250.0*i as f64),wx(1600.0),wy(192.0));

                wfd.generate_and_draw_texture(&mut target, win_w as u32, win_h as u32);
            }

            //###### MY DRAWING ENDS HERE ######
        target.finish().unwrap();

        event_loop.needs_update(); //force update
    }
    //#####################################################################################################################################################################################
}






// Generate a unique const `WidgetId` for each widget.
widget_ids!{
    struct Ids {
        canvas,
        button,
        file_navigator
    }
}
fn set_ui<'b,'a>(ref mut ui: conrod::UiCell, ids: &Ids, display: &'b glium::backend::glutin_backend::GlutinFacade, app: &mut AppState<'b>){
    #![allow(unused_imports)]
    use conrod::{color, widget, Colorable, Labelable, Positionable, Scalar, Sizeable, Widget};
    let win_w = ui.win_w;
    let win_h = ui.win_h;
    let dx = |d: f64|->f64{d*win_w/2.0};
    let dy = |d: f64|->f64{d*win_h/2.0};

    widget::Canvas::new()
        .color(conrod::color::DARK_CHARCOAL)
        .x_y(660.0,200.0)
        .w_h(600.0,600.0)
        .set(ids.canvas, ui);

    for _press in widget::Button::new()
        .label("Open File")
        .x_y(660.0,-130.0)
        .w_h(400.0, 50.0)
        .set(ids.button, ui)
        {
            println!("Pressed!");
            println!("{:?}", app.file_selection);

            if app.file_selection.is_some() {
                // ## load OPENBCI file
                println!("Reading OpenBCI data file.");
                let openbci_file=OpenBCIFile::new(app.file_selection.take().unwrap().to_str().unwrap());
                let wave_data = WaveData{
                    buffer: openbci_file.samples.clone(),
                    channels: openbci_file.channels,
                    sample_rate: 200,
                };
                app.wave_data=Some(wave_data);

                println!("Initialising waveform drawer.");
                app.waveform_drawers.clear();
                let wfwidth: u32=1320;
                let wfheight: u32=192;
                for i in 0..openbci_file.channels{
                app.waveform_drawers.push( WaveformDrawer::new( display,
                    WaveformDrawerSettings{
                            x: -300,
                            y: -250*i as i32 - wfheight as i32/2 + ui.win_h as i32/2,
                            width: wfwidth,
                            height: wfheight,
                            milliseconds_per_pixel: 2.5,
                            dtft_samples: 384,
                            channel: i}))
                }

                let ticks=app.ticker.ticks();
                for wfd in &mut app.waveform_drawers{
                    wfd.start(ticks);
                }
            }

        }


    let path = std::path::Path::new("data/");
    // Navigate the conrod directory only showing `.rs` and `.toml` files.
    for event in widget::FileNavigator::new(&path,conrod::widget::file_navigator::Types::All)
        .color(conrod::color::LIGHT_BLUE)
        .font_size(16)
        .wh_of(ids.canvas)
        .middle_of(ids.canvas)
        //.show_hidden_files(true)  // Use this to show hidden files
        .set(ids.file_navigator, ui)
        {
            use conrod::widget::file_navigator::Event;
            match event {
                Event::ChangeSelection(mut paths)=>
                    app.file_selection= if paths.len()>0 {Some(paths.pop().unwrap())} else {None},
                    _ => ()
            }
            //println!("{:?}", event);
        }



}
