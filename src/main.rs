extern crate log;
extern crate mlr_rs;
extern crate mds;
extern crate bela;
extern crate pretty_env_logger;
extern crate mbms_traits;
extern crate monome;

use std::thread;
use std::time::Duration;

use bela::*;
use mlr_rs::{MLR, MLRRenderer};
use mds::{MDS, MDSRenderer};
use mbms_traits::*;
use monome::*;

struct Control {
    controllers: Vec<Box<InstrumentControl>>,
    current: usize,
    monome: Monome
}

impl Control {
  fn new(monome: Monome) -> Control {
      Control {
          controllers: Vec::new(),
          current: 0,
          monome
      }
  }
  fn push(&mut self, controller: Box<InstrumentControl>) {
      self.controllers.push(controller);
  }
}

struct Render {
    renderers: Vec<Box<InstrumentRenderer>>
}

impl Render {
    fn new() -> Render {
        Render {
            renderers: Vec::new()
        }
    }
    fn push(&mut self, renderer: Box<InstrumentRenderer>) {
        self.renderers.push(renderer);
    }
}

struct MonomeTask<F> {
    callback: F,
    args: Control
}

impl<F> Auxiliary for MonomeTask<F>
where F: FnMut(&mut Control),
      for<'r> F: FnMut(&'r mut Control)
{
    type Args = Control;

    fn destructure(&mut self) -> (&mut FnMut(&mut Control), &mut Self::Args) {
        let MonomeTask {
            callback,
            args,
        } = self;

        (callback, args)
    }
}

type BelaApp<'a> = Bela<AppData<'a, Render>>;

fn go() -> Result<(), bela::error::Error> {
    println!("loading samples & decoding...");

    let (mlr, mlr_renderer) = MLR::new(BelaPort::AudioOut(0), 128., 44100);
    let (mlr2, mlr_renderer2) = MLR::new(BelaPort::AudioOut(1), 128., 44100);
    let (mds, mds_renderer) = MDS::new((BelaPort::AnalogOut(0), BelaPort::AnalogOut(7)), 16, 7, 128.);
    let monome = Monome::new("/prefix".to_string()).unwrap();

    let mut control = Control::new(monome);
    control.push(Box::new(mlr));
    control.push(Box::new(mlr2));
    control.push(Box::new(mds));

    let mut render = Render::new();
    render.push(Box::new(mlr_renderer));
    render.push(Box::new(mlr_renderer2));
    render.push(Box::new(mds_renderer));

    let mut monome_task = MonomeTask {
        callback: |control: &mut Control| {
            let mut grid = [0 as u8; 128];
            let mut last_grid = [0 as u8; 128];
            let monome = &mut control.monome;
            let controllers = &mut control.controllers;
            loop {
                match monome.poll() {
                    Some(e) => {
                        match e {
                            MonomeEvent::GridKey { x, y, direction: _ } => {
                                if y == 0 && x < 8 {
                                    control.current = x as usize;
                                } else {
                                    controllers[control.current].input(e);
                                }
                            },
                            _ => { }
                        }
                    }
                    _ => { }
                }
                for i in controllers.iter_mut() {
                    i.main_thread_work();
                }
                controllers[control.current].render(&mut grid);

                // light up the current instrument
                grid[control.current] = 15;

                let mut equal = true;
                for i in 0..128 {
                    if grid[i] != last_grid[i] {
                        equal = false;
                        break;
                    }
                }

                if !equal {
                    monome.set_all_intensity(&grid);
                    last_grid = grid;
                }

                grid.iter_mut().map(|x| *x = 0).count();

                thread::sleep(Duration::from_millis(32));
            }
        },
        args: control
    };

    let mut setup = |_context: &mut Context, _user_data: &mut Render| -> Result<(), error::Error> {
        println!("Setting up");
        let task = BelaApp::create_auxiliary_task(&mut monome_task, 10, "monome_task");
        BelaApp::schedule_auxiliary_task(&task)?;
        println!("ok");
        Ok(())
    };

    let mut cleanup = |_context: &mut Context, _user_data: &mut Render| {
        println!("Cleaning up");
    };

    let mut render_func = |context: &mut Context, renderers: &mut Render| {
        let insts = &mut renderers.renderers;
        for r in insts.iter_mut() {
            r.render(context);
        }
    };

    let user_data = AppData::new(render, &mut render_func, Some(&mut setup), Some(&mut cleanup));

    let mut settings = InitSettings::default();

    Bela::new(user_data).run(&mut settings)
}

fn main() {
    pretty_env_logger::init();

    match go() {
        Ok(_) => { println!("??"); }
        Err(_) => { println!("!!"); }
    }
}
