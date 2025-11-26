use jni::{InitArgsBuilder, JNIEnv, JavaVM, objects::JValueGen};

pub use mcje_macros::*;

pub async fn init() -> JavaVM {
    let classpath = env!("MCJE_JVM_CLASSPATH");

    let jvm_args = InitArgsBuilder::new()
        .option(format!("-Djava.class.path={classpath}"))
        .build()
        .unwrap();

    JavaVM::new(jvm_args).unwrap()
}

pub fn bootstrap(env: &mut JNIEnv) {
    let detected_version_built_in = env
        .get_static_field(
            "net/minecraft/DetectedVersion",
            "BUILT_IN",
            "Lnet/minecraft/WorldVersion;",
        )
        .unwrap()
        .l()
        .unwrap();

    env.call_static_method(
        "net/minecraft/SharedConstants",
        "setVersion",
        "(Lnet/minecraft/WorldVersion;)V",
        &[JValueGen::Object(&detected_version_built_in)],
    )
    .unwrap();

    env.call_static_method("net/minecraft/server/Bootstrap", "bootStrap", "()V", &[])
        .unwrap();
}
