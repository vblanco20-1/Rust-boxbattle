//! Basic hello world example.

#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate ggez;
extern crate sdl2;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate rand;


extern crate collision;
extern crate ncollide as nc;
extern crate nalgebra as na;


use nc::shape::Cuboid;
use nc::query::{Ray, RayCast,RayIntersection};



use std::sync::Arc;
use ggez::conf;
use ggez::event::{self, MouseButton, Button, MouseState, Keycode, Mod, Axis};
use ggez::{Context, GameResult};
use ggez::graphics;
use std::env;
use std::path;
use rand::{Rng, thread_rng};
use sdl2::mouse;
use specs::*;
use ggez::graphics::*;
use std::cell::UnsafeCell;

#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Motion {
    pub velocity: Vector2,
    pub acceleration: Vector2,
}

#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Position(Vector2);

#[derive(Clone, Debug, Component)]
#[component(HashMapStorage)]
pub struct MovementInput(Vector2);

#[derive(Clone, Debug, Component)]
#[component(HashMapStorage)]
pub struct FirePoint{
    pub offset: Vector2,
    pub fire_rate: f32,
    pub to_next_shot: f32
}

impl FirePoint {
    fn new() -> FirePoint { FirePoint{offset: Vector2::new(0.0,0.0),fire_rate:0.2,to_next_shot: 0.0}}
}

#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct Rect {
    pub size: Vector2
}


#[derive(Default)]
pub struct Ball;
impl Component for Ball { type Storage = NullStorage<Self>;}
#[derive(Default)]
pub struct Killable;
impl Component for Killable { type Storage = NullStorage<Self>;}

#[derive(Clone, Debug, Component)]
#[component(VecStorage)]
pub struct RenderColor(graphics::Color) ;


struct ContextWrapper{

pub ctx: *mut Context

}

impl ContextWrapper{
    fn get(&self) -> &mut Context
    {
        unsafe{
            &mut *self.ctx
        }
    }
}
struct DeltaTime(f32);



unsafe impl Sync for ContextWrapper{}

unsafe impl Send for ContextWrapper{}

#[derive(Clone, Copy)]
struct PlayerInput{
    move_x: f32,
    fire: bool
}
impl PlayerInput{
    fn new()->PlayerInput{
        PlayerInput{move_x: 0.0, fire:false}
    }
}

struct RectangleRenderSystem;



impl<'a> specs::System<'a> for RectangleRenderSystem{
    type SystemData = ( ReadStorage<'a,RenderColor>,ReadStorage<'a,Rect>,ReadStorage<'a,Position>,Fetch<'a,ContextWrapper> );

    fn run(&mut self, (col,rect,pos,_ctx): Self::SystemData) {
        let ctx: &mut Context;
        unsafe{
             ctx = &mut *_ctx.ctx;
        }
        let original_color = graphics::get_color(ctx);

        let mb = &mut graphics::MeshBuilder::new();


        let offsety = 0.0;
        let offsetx = 0.0;

        //build box
        mb.polygon( graphics::DrawMode::Fill,
            &[
                Point2::new(offsetx - 0.5,offsety- 0.5),
                Point2::new(offsetx + 0.5,offsety- 0.5),
                Point2::new(offsetx + 0.5,offsety+ 0.5),
                Point2::new(offsetx -0.5,offsety+ 0.5),
                //Point2::new(200.0, 300.0),
            ]

        );
        let mut mesh = mb.build(ctx);

        let umesh = mesh.unwrap();


        for (c,r,p) in (&col,&rect,&pos).join() {

            graphics::set_color(ctx,c.0);
            let min:Vector2 = p.0 - r.size/2.0;

            let drawpar = graphics::DrawParam {

                dest: Point2::new(p.0.x , p.0.y ),
                scale: Point2::new(r.size.x,r.size.y),

                ..Default::default()
            };
            graphics::draw_ex(ctx, &umesh, drawpar);


        }
        graphics::set_color(ctx,original_color);
    }
}
struct MotionSystem;

impl<'a> specs::System<'a> for MotionSystem {
    type SystemData = (WriteStorage<'a, Position>, WriteStorage<'a, Motion>,FetchMut<'a, DeltaTime>);

    fn run(&mut self, (mut position,mut motion,deltatime): Self::SystemData) {

        let dt:f32 = deltatime.0;

        for (pos,mot) in (&mut position,&mut motion).join() {

            mot.velocity += mot.acceleration * dt;
            pos.0 += mot.velocity * dt;
        }

    }
}

struct PlayerInputSystem;

impl<'a> specs::System<'a> for PlayerInputSystem {

    type SystemData = (  Entities<'a>,WriteStorage<'a, MovementInput>,WriteStorage<'a, Motion>,Fetch<'a,PlayerInput> ,Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities,mut input,mut motion,playerinput, updater): Self::SystemData) {


        for (n,mt) in (&mut input,&mut motion).join() {

            n.0.x = playerinput.move_x;
            mt.velocity.x = playerinput.move_x * 300.0;
        }
    }
}

struct FirePointSystem;

impl<'a> specs::System<'a> for FirePointSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, Position>, WriteStorage<'a, FirePoint>, Fetch<'a, PlayerInput>,Fetch<'a, DeltaTime>, Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, position, mut firepoint, playerinput,delta, updater): Self::SystemData) {
        for (pos, fp) in (&position, &mut firepoint).join() {
            fp.to_next_shot -= delta.0;
            if playerinput.fire {

                let shotpos = pos.0 + fp.offset;


                if fp.to_next_shot < 0.0
                {
                    fp.to_next_shot = fp.fire_rate;
                    println!("FIRE");
                    let mut rng = rand::thread_rng();


                    let roll:f32 = rng.gen();

                    let fire = entities.create();
                    updater.insert(fire,Position(shotpos));
                    updater.insert(fire,Rect{ size: Vector2::new(10.0,20.0)});
                    updater.insert(fire,RenderColor(graphics::Color::new(1.0,0.0 ,0.0,1.0) ));
                    updater.insert(fire,Motion{velocity: Vector2::new((roll - 0.5)*400.0 ,-400.0), acceleration: Vector2::new(0.0,0.0)  });
                    updater.insert(fire,Ball);
                }
            }
        }
    }
}



fn rectangle_collision(originA: Vector2,sizeA: Vector2,originB: Vector2,sizeB: Vector2) -> bool{

   let RectA = graphics::Rect::new(originA.x-sizeA.x/2.0, originA.y-sizeA.y/2.0,sizeA.x,sizeA.y  );
    let RectB = graphics::Rect::new(originB.x-sizeB.x/2.0, originB.y-sizeB.y/2.0,sizeB.x,sizeB.y  );

    return RectA.overlaps(&RectB);
}

struct HitResult{
    hit_point:Vector2,
    hit_normal:Vector2
}

fn ray_rectangle_collision(boxOrigin: Vector2,boxSize: Vector2, rayorigin:Vector2, raydirection:Vector2) -> Option<HitResult>
{

    let cuboid = Cuboid::new( na::Vector2::new(boxSize.x, boxSize.y) *0.5);

    let ray_origin =  boxOrigin - rayorigin;
    let ro = na::Point2::new(ray_origin.x,ray_origin.y);
    let rd = na::Vector2::new(raydirection.x,raydirection.y);
    let ray = Ray::new(ro, rd);

   let hit =  cuboid.toi_and_normal_with_ray(&na::Id::new(),&ray, true);


    //return hit;
    //return !hit.is_none();
    match hit {
        // The division was valid
        Some(result) => {

            if result.toi >= 0.01
                {
                    let intersection_point = ro + rd* result.toi;
                    let hitpoint : Vector2 = Vector2::new(intersection_point.x,intersection_point.y) ;
                    let hitnormal : Vector2 = Vector2::new(result.normal.x, result.normal.y);

                    return Some( HitResult{hit_point:hitpoint, hit_normal: hitnormal  }   ) ;
                }



        },
        None    => { return None},
    }


    return None
}


struct BallInfo{
    position: Vector2,
    extent: Vector2,
    direction: Vector2,
    ent: Entity
}
struct BallCollisionSystem;
impl<'a> specs::System<'a> for BallCollisionSystem {



    type SystemData = (Entities<'a>, ReadStorage<'a, Position>,WriteStorage<'a, Motion> ,ReadStorage<'a, Rect>,ReadStorage<'a, Ball>,ReadStorage<'a, Killable> ,Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, position,motion, rectangle, ball,killable, updater): Self::SystemData) {

        let mut balls = Vec::with_capacity(10);
        for (entity,pos,mt, rect,ball) in (&*entities,&position,&motion, &rectangle,&ball).join() {

            balls.push(BallInfo{ position:pos.0, extent: rect.size,ent:entity,direction: mt.velocity  });

        }

        for (entity,pos, rect,killable) in (&*entities,&position, &rectangle,&killable).join() {

            for b in &balls{

                let hit = ray_rectangle_collision(pos.0,rect.size, b.position, b.direction);

                match hit {
                    // The division was valid
                    Some(x) =>{
                        println!("Hit {}", x.hit_normal);

                        updater.insert(b.ent,Motion{velocity:  x.hit_normal * 200.0, acceleration: Vector2::new(0.0,0.0)  });
                       // entities.delete(b.ent);
//
                        //let intersection_point = ;
//
                        //let fire = entities.create();
                        //updater.insert(fire,Position(x.hit_point));
                        //updater.insert(fire,Rect{ size: Vector2::new(10.0,10.0)});
                        //updater.insert(fire,RenderColor(graphics::Color::new(1.0,1.0 ,0.0,1.0) ));
                       // updater.insert(fire,Motion{velocity: Vector2::new((roll - 0.5)*400.0 ,-400.0), acceleration: Vector2::new(0.0,0.0)  });
                        //updater.insert(fire,Ball);


                    },
                    // The division was invalid
                    None    => {},
                }
                //if hit {
                //    println!("HIT");
                //}

                //let collisionbox: Cuboid2<f32> = Cuboid2::new( rect.size) ;// ncollide::ncollide_math::Vector  ::new(rect.size.x * 0.5 ,rect.size.y * 0.5 )   );
                //let ray = Ray::new(    );

               if rectangle_collision(b.position,b.extent,pos.0,rect.size) {
                   println!("OVERLAPS");
                   entities.delete(entity);

               }
            }
        }
    }
}


// First we make a structure to contain the game's state
struct MainState {
    pub text: graphics::Text,
    pub frames: usize,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub input: PlayerInput,
    pub specs_world: specs::World,
    pub specs_dispatcher: specs::Dispatcher<'static, 'static>
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {

        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.
        let font = graphics::Font::new(_ctx, "/DejaVuSerif.ttf", 48)?;
        let text = graphics::Text::new(_ctx, "EXPERIMENT", &font)?;

        let mut w = specs::World::new();
        w.register::<Position>();
        w.register::<Motion>();
        w.register::<MovementInput>();
        w.register::<RenderColor>();
        w.register::<Rect>();
        w.register::<FirePoint>();
        w.register::<Ball>();
        w.register::<Killable>();

        //create bricks
        for x in 0..10 {
            for y in 0..10 {
                let fx = x as f32;
                let fy = y as f32;
                w.create_entity()
                    .with(Position(Vector2::new(fx * 55.0 + 100.0,fy * 25.0 + 100.0)))
                    .with(Rect{ size: Vector2::new(50.0,20.0)})
                    .with(RenderColor(graphics::Color::new(1.0,fx /10.0,fy / 10.0,1.0) ))
                    .with(Killable)
                    //.with(MovementInput(Vector2::new(0.0,0.0)))
                    //.with(Motion{velocity: Vector2::new(0.0,0.0), acceleration: Vector2::new(0.0,0.0)  })
                    .build();
            }
        }


        //create player
        w.create_entity()
            .with(Position(Vector2::new(300.0,500.0)))
            .with(Rect{ size: Vector2::new(90.0,20.0)})
            .with(RenderColor(graphics::Color::new(1.0,1.0,1.0,1.0) ))
            .with(MovementInput(Vector2::new(0.0,0.0)))
            .with(Motion{velocity: Vector2::new(0.0,0.0), acceleration: Vector2::new(0.0,0.0)  })
            .with(FirePoint::new())
            .build();



        let ctxwrapper = ContextWrapper{
            ctx: _ctx as *mut Context
        };
        w.add_resource(ctxwrapper);
        w.add_resource(DeltaTime(0.5));
        w.add_resource(PlayerInput::new());


        // ...oooooh, the dispatcher should go in the Scene
        // so every scene can have its own set of systems!
        let dispatcher = specs::DispatcherBuilder::new()
           // .add_thread_local(MotionSystem)
            .add_thread_local(RectangleRenderSystem)
            .add(PlayerInputSystem,"input_sys",&[])
            .add(MotionSystem,"motion_sys" ,&["input_sys"])
            .add(FirePointSystem,"fire_sys",&[])
            .add(BallCollisionSystem,"ball_sys",&["motion_sys"])

            .build();


        let s = MainState {
            text,
            frames: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            input: PlayerInput::new(),
            specs_world: w,
            specs_dispatcher: dispatcher
        };
        Ok(s)
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        let dt: f32 = ggez::timer::duration_to_f64(ggez::timer::get_delta(ctx)) as f32;

        {
        let mut delta = self.specs_world.write_resource::<DeltaTime>();
        *delta = DeltaTime(dt);

            let mut pi = self.specs_world.write_resource::<PlayerInput>();
            *pi = self.input;
         }


        let r = graphics::rectangle(ctx,graphics::DrawMode::Fill ,graphics::Rect::new(self.mouse_x,self.mouse_y,5.0,5.0) )?;

        // Drawables are drawn from their top-left corner.
        let dest_point = graphics::Point2::new(0.0,0.0);
        graphics::draw(ctx, &self.text, dest_point, 0.0)?;

        self.specs_dispatcher.dispatch(&self.specs_world.res);
        self.specs_world.maintain();

        graphics::present(ctx);

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::get_fps(ctx));
        }

        Ok(())
    }
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, keymod: Mod, repeat: bool) {

        if !repeat{
            match keycode{
                Keycode::Right => {self.input.move_x += 1.0;},
                Keycode::Left => {self.input.move_x -= 1.0;},
                Keycode::Down => {},
                Keycode::Up => {},
                Keycode::Space => {self.input.fire = true},
                _ => {}
            }
        }
        println!(
            "Key pressed: {:?}, modifier {:?}, repeat: {}",
            keycode, keymod, repeat
        );
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, keymod: Mod, repeat: bool) {
        if !repeat{
            match keycode{
                Keycode::Right => {self.input.move_x -= 1.0;},
                Keycode::Left => {self.input.move_x += 1.0;},
                Keycode::Down => {},
                Keycode::Up => {},
                Keycode::Space => {self.input.fire = false},
                _ => {}
            }
        }
        println!(
            "Key released: {:?}, modifier {:?}, repeat: {}",
            keycode, keymod, repeat
        );
    }
    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _state: mouse::MouseState,
        _x: i32,
        _y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        self.mouse_x = _x as f32;
        self.mouse_y = _y as f32;
    }
}

// Now our main function, which does three things:
//
// * First, create a new `ggez::conf::Conf`
// object which contains configuration info on things such
// as screen resolution and window title.
// * Second, create a `ggez::game::Game` object which will
// do the work of creating our MainState and running our game.
// * Then, just call `game.run()` which runs the `Game` mainloop.
pub fn main() {
    let mut c = conf::Conf::new();

    let ctx = &mut Context::load_from_conf("helloworld", "ggez", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources to the filesystem's path
    // so that ggez will look in our cargo project directory for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}