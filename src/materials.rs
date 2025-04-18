use crate::Material;
use crate::{Vec3f, Vec4f};

const RED_MATERIAL_ALBEDO: Vec4f = Vec4f::const_new_with_data([0.6, 0.3, 0.0, 0.1]);
const RED_MATERIAL_DIFFUSE_COLOR: Vec3f = Vec3f::const_new_with_data([1.0, 0.1, 0.1]);
const RED_MATERIAL_AMBIENT_COLOR: Vec3f = Vec3f::const_new_with_data([0.2, 0.05, 0.05]);

pub const RED_MATERIAL: Material = Material::const_new(
    RED_MATERIAL_ALBEDO,
    RED_MATERIAL_DIFFUSE_COLOR,
    RED_MATERIAL_AMBIENT_COLOR,
    250.0,
    1.0,
);

const GREEN_MATERIAL_ALBEDO: Vec4f = Vec4f::const_new_with_data([0.6, 0.3, 0.0, 0.1]);
const GREEN_MATERIAL_DIFFUSE_COLOR: Vec3f = Vec3f::const_new_with_data([0.1, 1.0, 0.1]);
const GREEN_MATERIAL_AMBIENT_COLOR: Vec3f = Vec3f::const_new_with_data([0.05, 0.2, 0.05]);

pub const GREEN_MATERIAL: Material = Material::const_new(
    GREEN_MATERIAL_ALBEDO,
    GREEN_MATERIAL_DIFFUSE_COLOR,
    GREEN_MATERIAL_AMBIENT_COLOR,
    125.0,
    1.0,
);

const BLUE_MATERIAL_ALBEDO: Vec4f = Vec4f::const_new_with_data([0.6, 0.3, 0.0, 0.1]);
const BLUE_MATERIAL_DIFFUSE_COLOR: Vec3f = Vec3f::const_new_with_data([0.1, 0.1, 1.0]);
const BLUE_MATERIAL_AMBIENT_COLOR: Vec3f = Vec3f::const_new_with_data([0.05, 0.05, 0.2]);

pub const BLUE_MATERIAL: Material = Material::const_new(
    BLUE_MATERIAL_ALBEDO,
    BLUE_MATERIAL_DIFFUSE_COLOR,
    BLUE_MATERIAL_AMBIENT_COLOR,
    125.0,
    1.0,
);

const MIRROR_MATERIAL_ALBEDO: Vec4f = Vec4f::const_new_with_data([0.0, 0.0, 0.9, 0.03]);
const MIRROR_MATERIAL_DIFFUSE_COLOR: Vec3f = Vec3f::const_new_with_data([1.0, 1.0, 1.0]);
const MIRROR_MATERIAL_AMBIENT_COLOR: Vec3f = Vec3f::const_new_with_data([0.0, 0.0, 0.0]);

pub const MIRROR_MATERIAL: Material = Material::const_new(
    MIRROR_MATERIAL_ALBEDO,
    MIRROR_MATERIAL_DIFFUSE_COLOR,
    MIRROR_MATERIAL_AMBIENT_COLOR,
    1000.0,
    1.0,
);

const GLASS_MATERIAL_ALBEDO: Vec4f = Vec4f::const_new_with_data([0.0, 0.5, 0.1, 0.8]);
const GLASS_MATERIAL_DIFFUSE_COLOR: Vec3f = Vec3f::const_new_with_data([0.6, 0.7, 0.8]);
const GLASS_MATERIAL_AMBIENT_COLOR: Vec3f = Vec3f::const_new_with_data([0.1, 0.1, 0.2]);

pub const GLASS_MATERIAL: Material = Material::const_new(
    GLASS_MATERIAL_ALBEDO,
    GLASS_MATERIAL_DIFFUSE_COLOR,
    GLASS_MATERIAL_AMBIENT_COLOR,
    300.0,
    1.5,
);

const GOLD_MATERIAL_ALBEDO: Vec4f = Vec4f::const_new_with_data([0.8, 0.3, 0.0, 0.0]);
const GOLD_MATERIAL_DIFFUSE_COLOR: Vec3f = Vec3f::const_new_with_data([1.0, 0.843, 0.0]);
const GOLD_MATERIAL_AMBIENT_COLOR: Vec3f = Vec3f::const_new_with_data([0.2, 0.17, 0.05]);

pub const GOLD_MATERIAL: Material = Material::const_new(
    GOLD_MATERIAL_ALBEDO,
    GOLD_MATERIAL_DIFFUSE_COLOR,
    GOLD_MATERIAL_AMBIENT_COLOR,
    500.0,
    1.0,
);
