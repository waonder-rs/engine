# Engine

## Object

Each object is identified.

### Components

#### Hierarchy

Parent/child relation.

#### Transformation

Matrix

#### Geometry

Each geometry is identified.

- Visible
- Collision

#### Material

Each material is identified.

Description

## Pipeline

### Stages

#### Commands

| Read | Write |
| ---- | ----- |
| all  | all   |

#### Logic

| Read | Write |
| ---- | ----- |
| all  | all   |

#### Physics

| Read                                            | Write          |
| ----------------------------------------------- | -------------- |
| hierarchy, transformation, geometry (collision) | transformation |

#### Renderer

Attached to a device.

| Read                                                     | Write         |
| -------------------------------------------------------- | ------------- |
| hierarchy, transformations, geometry (display), material | vulkan device |