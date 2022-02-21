//! 2-dimensional coordinates

use crate::orientation::{
    Direction, NearlySingularConversion, OrientationPositionInterop, Rotation,
};
use bevy_ecs::prelude::Component;
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, Error, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign,
};
use std::{fmt::Debug, ops::*};

/// A float could not be converted into a [`Coordinate`]
#[derive(Debug, Clone, Copy, Error, Display, PartialEq, Eq)]
pub struct FloatCoordinateConversionError;

/// A 2-dimensional coordinate
///
/// The underlying data type `T` can be modified to control
/// whether the coordinate system is hexagonal vs. grid,
/// continuous or discrete and so on.
///
/// # Example
/// ```rust
/// use leafwing_2d::position::Position;
/// let origin = Position::default();
/// let player_position = Position::<f32>::new(10.0, 4.0);
///
/// assert_eq!(player_position + origin, player_position);
/// assert_eq!(player_position - origin, player_position);
/// ```
#[derive(
    Component,
    Default,
    Clone,
    Copy,
    Debug,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    PartialEq,
)]
pub struct Position<C: Coordinate> {
    /// The first coordinate of the position, typically the x-axis
    pub x: C,
    /// The second coordinate of the position, typically the y-axis
    pub y: C,
}

impl<C: Coordinate> Position<C> {
    /// Creates a new [`Position`] with the provided `x` and `y` coordinates
    #[inline]
    #[must_use]
    pub fn new(x: C, y: C) -> Position<C> {
        Position { x, y }
    }
}

/// A type that can be used as a coordinate type for [`Position`]
///
/// This trait has a blanket impl for all types that impl [`TryFrom<f32>`], and so is already implemented for all of the base float types.
///
/// If you are working with a grid-like position system, use one of the types provided in [`discrete_coordinates`] type
/// (or your own [`DiscreteCoordinate`](discrete_coordinates::DiscreteCoordinate) type),
/// rather than a raw integer type.
/// We cannot impl this trait for those types due to conflicting impls,
/// and additional functionality is provided by the [`DiscreteCoordinate`](discrete_coordinates::DiscreteCoordinate) trait.
pub trait Coordinate:
    Copy
    + Debug
    + Default
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Rem<Output = Self>
    + RemAssign
    + PartialOrd
    + Send
    + Sync
    + Into<f32>
    + 'static
{
    /// Attempt to create a [`Coordinate`] from a `f32`, as might be returned by [`Transform`](bevy_transform::components::Transform)
    fn try_from_f32(float: f32) -> Result<Self, FloatCoordinateConversionError>;
}

impl<T> Coordinate for T
where
    T: Copy
        + Debug
        + Default
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + SubAssign
        + Mul<Output = Self>
        + MulAssign
        + Div<Output = Self>
        + DivAssign
        + Rem<Output = Self>
        + RemAssign
        + PartialOrd
        + Send
        + Sync
        + Into<f32>
        + TryFrom<f32>
        + 'static,
{
    fn try_from_f32(float: f32) -> Result<Self, FloatCoordinateConversionError> {
        let result = float.try_into();

        match result {
            Ok(coordinate) => Ok(coordinate),
            Err(_) => Err(FloatCoordinateConversionError),
        }
    }
}

impl<C: Coordinate> Position<C> {
    /// Gets the [`Direction`] that points away from this position, towards `other_position`
    ///
    /// # Example
    /// ```rust
    /// use leafwing_2d::position::Position;
    /// use leafwing_2d::orientation::Direction;
    ///
    /// let origin = Position::<f32>::default();
    /// let target = Position::new(0.0, 1.0);
    ///
    /// assert_eq!(origin.direction_to(target), Ok(Direction::NORTH));
    /// ```
    #[inline]
    #[must_use]
    pub fn direction_to(
        self,
        other_position: Position<C>,
    ) -> Result<Direction, NearlySingularConversion> {
        let net_position: Position<C> = other_position - self;
        net_position.try_into()
    }

    /// Gets the [`Direction`] that points towards this position, from `other_position`
    ///
    /// # Example
    /// ```rust
    /// use leafwing_2d::position::Position;
    /// use leafwing_2d::orientation::Direction;
    ///
    /// let origin = Position::<f32>::default();
    /// let target = Position::new(0.0, 1.0);
    ///
    /// assert_eq!(origin.direction_from(target), Ok(Direction::SOUTH));
    /// ```
    #[inline]
    #[must_use]
    pub fn direction_from(
        self,
        other_position: Position<C>,
    ) -> Result<Direction, NearlySingularConversion> {
        let net_position: Position<C> = self - other_position;
        net_position.try_into()
    }

    /// Gets the [`Rotation`] that points away from this position, towards `other_position`
    ///
    /// # Example
    /// ```rust
    /// use leafwing_2d::position::Position;
    /// use leafwing_2d::orientation::Rotation;
    ///
    /// let origin = Position::<f32>::default();
    /// let target = Position::new(0.0, 1.0);
    ///
    /// assert_eq!(origin.rotation_to(target), Ok(Rotation::NORTH));
    /// ```
    #[inline]
    #[must_use]
    pub fn orientation_to<O: OrientationPositionInterop<C>>(
        self,
        other_position: Position<C>,
    ) -> Result<O, NearlySingularConversion> {
        O::orientation_to_position(self, other_position)
    }

    /// Gets the [`Rotation`] that points towards this position, from `other_position`
    ///
    /// # Example
    /// ```rust
    /// use leafwing_2d::position::Position;
    /// use leafwing_2d::orientation::Rotation;
    ///
    /// let origin = Position::<f32>::default();
    /// let target = Position::new(0.0, -1.0);
    ///
    /// assert_eq!(origin.rotation_from(target), Ok(Rotation::NORTH));
    /// ```
    #[inline]
    #[must_use]
    pub fn rotation_from(
        self,
        other_position: Position<C>,
    ) -> Result<Rotation, NearlySingularConversion> {
        let net_position: Position<C> = self - other_position;
        net_position.try_into()
    }
}

/// Coordinate types for [`Position`] designed for operation on discrete grids
///
/// The provided types all store an `isize` under the hood for maximum flexbility.
/// If you require a different storage type,
/// please feel free to copy-paste the relevant struct def and trait impls into your game
/// and modify `isize` to your desired integer type.
pub mod discrete_coordinates {
    use crate::orientation::{partitioning::DirectionParitioning, Direction};

    use super::*;

    /// A type that can be used to represent a discrete 2-dimensional coordinate
    ///
    /// Typically used to create and work with [`Positions`](Position)
    pub trait DiscreteCoordinate: Coordinate {
        /// The number of neighbors
        const N_NEIGHBORS: usize;
        /// Adding or subtracting this coordinate to another coordinate does not change the value
        const ZERO: Self;
        /// The (0, 0) cell [`Position`]
        const ORIGIN: Position<Self> = Position {
            x: Self::ZERO,
            y: Self::ZERO,
        };

        /// The [`DirectionParitioning`] that determines how [`Directions`](Direction) should map to neighbors
        type Parititions: DirectionParitioning;

        /// Creates a [`Position`] from the pair of values provided
        #[must_use]
        fn position(x: Self, y: Self) -> Position<Self> {
            Position { x, y }
        }

        /// Fetches the array of neighboring [`Positions`](Position), in a fixed order
        ///
        /// The order should always be clockwise, starting from north (+y)
        #[must_use]
        fn neighbors(position: Position<Self>) -> [Position<Self>; Self::N_NEIGHBORS];

        /// The [`Direction`] towards each neighbor
        ///
        /// The order should always be clockwise, starting from north (+y)
        ///
        /// ```rust
        ///
        ///
        /// ```
        #[must_use]
        fn neighbor_directions() -> [Direction; Self::N_NEIGHBORS] {
            Self::neighbors(Self::ORIGIN).map(|position| {
                position
                    .try_into()
                    .expect("The positions of the neighbors cannot be (0,0).")
            })
        }
    }

    /// [`DiscreteCoordinate`] primitive for a square grid, where each cell has four neighbors
    ///
    /// Neighboring tiles must touch on their faces
    #[derive(
        Clone,
        Copy,
        Debug,
        Add,
        AddAssign,
        Sub,
        SubAssign,
        Rem,
        RemAssign,
        Mul,
        MulAssign,
        Div,
        DivAssign,
        PartialEq,
        Default,
        PartialOrd,
    )]
    pub struct OrthogonalGrid(pub isize);

    /*
    impl DiscreteCoordinate for OrthogonalGrid {
        const N_NEIGHBORS: usize = 4;
        const ZERO: OrthogonalGrid = OrthogonalGrid(0);
        type Parititions = CardinalQuadrant;

        fn neighbors(position: Position<Self>) -> [Position<Self>; Self::N_NEIGHBORS] {
            [
                Position {
                    x: Self(position.x.0),
                    y: Self(position.y.0 + 1),
                },
                Position {
                    x: Self(position.x.0 + 1),
                    y: Self(position.y.0),
                },
                Position {
                    x: Self(position.x.0),
                    y: Self(position.y.0 - 1),
                },
                Position {
                    x: Self(position.x.0 - 1),
                    y: Self(position.y.0),
                },
            ]
        }
    }
    */

    impl From<OrthogonalGrid> for f32 {
        fn from(coordinate: OrthogonalGrid) -> f32 {
            coordinate.0 as f32
        }
    }

    impl From<f32> for OrthogonalGrid {
        fn from(float: f32) -> OrthogonalGrid {
            OrthogonalGrid(float.round() as isize)
        }
    }

    /// [`DiscreteCoordinate`] primitive for a square grid, where each cell has eight neighbors
    ///
    /// Neighboring tiles are a king's move away: either touching faces or diagonally adjacent
    pub struct AdjacentGrid(pub isize);

    /// [`DiscreteCoordinate`] primitive for a hexagonal grid, where each cell points sideways
    ///
    /// These hexes tile vertically, but not horizontally
    pub struct FlatHex(pub isize);

    /// [`DiscreteCoordinate`] primitive for a hexagonal grid, where each cell points up
    ///
    /// These hexes tile horizontally, but not vertically
    pub struct PointyHex(pub isize);
}

mod basic_operations {
    use super::*;

    impl<C: Coordinate> Add<Position<C>> for Position<C> {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
            }
        }
    }

    impl<C: Coordinate> Sub<Position<C>> for Position<C> {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            }
        }
    }

    impl<C: Coordinate> Mul<C> for Position<C> {
        type Output = Position<C>;

        fn mul(self, rhs: C) -> Self::Output {
            Self {
                x: self.x * rhs,
                y: self.y * rhs,
            }
        }
    }

    impl<C: Coordinate> Div<C> for Position<C> {
        type Output = Position<C>;

        fn div(self, rhs: C) -> Self::Output {
            Self {
                x: self.x / rhs,
                y: self.y / rhs,
            }
        }
    }

    impl<C: Coordinate> Rem<C> for Position<C> {
        type Output = Position<C>;

        fn rem(self, rhs: C) -> Self::Output {
            Self {
                x: self.x % rhs,
                y: self.y % rhs,
            }
        }
    }

    impl<C: Coordinate> Rem<Position<C>> for Position<C> {
        type Output = Position<C>;

        fn rem(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x % rhs.x,
                y: self.y % rhs.y,
            }
        }
    }
}

// When relevant, z-values are simply ignored
mod conversions {
    use super::*;
    use crate::orientation::Direction;
    use bevy_math::{Quat, Vec2, Vec3};
    use bevy_transform::components::{GlobalTransform, Transform};

    impl<C: Coordinate> TryFrom<Vec2> for Position<C> {
        type Error = FloatCoordinateConversionError;

        fn try_from(vec: Vec2) -> Result<Position<C>, FloatCoordinateConversionError> {
            let x = C::try_from_f32(vec.x)?;
            let y = C::try_from_f32(vec.y)?;

            Ok(Position { x, y })
        }
    }

    impl<C: Coordinate> From<Position<C>> for Vec2 {
        fn from(position: Position<C>) -> Vec2 {
            Vec2::new(position.x.into(), position.y.into())
        }
    }

    impl<C: Coordinate> From<Position<C>> for Vec3 {
        fn from(position: Position<C>) -> Vec3 {
            Vec3::new(position.x.into(), position.y.into(), 0.0)
        }
    }

    impl<C: Coordinate> TryFrom<Position<C>> for Direction {
        type Error = NearlySingularConversion;

        fn try_from(position: Position<C>) -> Result<Direction, NearlySingularConversion> {
            let vec2: Vec2 = position.into();

            vec2.try_into()
        }
    }

    impl<C: Coordinate> TryFrom<Position<C>> for Rotation {
        type Error = NearlySingularConversion;

        fn try_from(position: Position<C>) -> Result<Rotation, NearlySingularConversion> {
            let vec2: Vec2 = position.into();

            vec2.try_into()
        }
    }

    impl<C: Coordinate> TryFrom<Position<C>> for Quat {
        type Error = NearlySingularConversion;

        fn try_from(position: Position<C>) -> Result<Quat, NearlySingularConversion> {
            let direction: Direction = position.try_into()?;

            Ok(direction.into())
        }
    }

    impl<C: Coordinate> TryFrom<Vec3> for Position<C> {
        type Error = FloatCoordinateConversionError;

        fn try_from(vec: Vec3) -> Result<Position<C>, FloatCoordinateConversionError> {
            let x = C::try_from_f32(vec.x)?;
            let y = C::try_from_f32(vec.y)?;

            Ok(Position { x, y })
        }
    }

    impl<C: Coordinate> TryFrom<Transform> for Position<C> {
        type Error = FloatCoordinateConversionError;

        fn try_from(transform: Transform) -> Result<Position<C>, FloatCoordinateConversionError> {
            let x = C::try_from_f32(transform.translation.x)?;
            let y = C::try_from_f32(transform.translation.y)?;

            Ok(Position { x, y })
        }
    }

    impl<C: Coordinate> TryFrom<GlobalTransform> for Position<C> {
        type Error = FloatCoordinateConversionError;

        fn try_from(
            transform: GlobalTransform,
        ) -> Result<Position<C>, FloatCoordinateConversionError> {
            let x = C::try_from_f32(transform.translation.x)?;
            let y = C::try_from_f32(transform.translation.y)?;

            Ok(Position { x, y })
        }
    }
}
