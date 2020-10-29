use cgperspective::{
    Camera,
    PerspectiveFovProjection,
    FreeKinematics,
};


pub type PerspectiveFovCamera<S> = Camera<S, PerspectiveFovProjection<S>, FreeKinematics<S>>;

