pub mod world;

mod extension;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod testsa {
    use bevy::prelude::App;
    use deno_core::JsRuntime;

    #[test]
    fn test1() {
        let app = App::new();

        let mut runtime = JsRuntime::new(Default::default());
    }
}
