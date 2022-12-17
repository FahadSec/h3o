use crate::{
    coord::{CoordIJK, FaceIJK},
    error, Direction, Face, NUM_PENTAGONS, NUM_PENT_VERTS,
};
use std::fmt;

/// Maximum value for a base cell.
pub const MAX: u8 = 121;

// Bitmap where a bit's position represents a base cell value.
const BASE_PENTAGONS: u128 = 0x0020_0802_0008_0100_8402_0040_0100_4010;

// -----------------------------------------------------------------------------

/// One of the 122 base cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct BaseCell(u8);

impl BaseCell {
    /// Initializes a new base cell using a value that may be out of range.
    ///
    /// # Safety
    ///
    /// The value must be a valid base cell.
    pub(crate) const fn new_unchecked(value: u8) -> Self {
        debug_assert!(value <= MAX, "base cell out of range");
        Self(value)
    }

    /// Returns true if the base cell is pentagonal.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::BaseCell;
    ///
    /// assert!(BaseCell::try_from(4)?.is_pentagon());
    /// assert!(!BaseCell::try_from(8)?.is_pentagon());
    /// # Ok::<(), h3o::error::InvalidBaseCell>(())
    /// ```
    #[must_use]
    pub const fn is_pentagon(self) -> bool {
        BASE_PENTAGONS & (1 << self.0) != 0
    }

    /// Returns the total number of base cells.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::BaseCell;
    ///
    /// assert_eq!(BaseCell::count(), 122);
    /// ```
    #[must_use]
    pub const fn count() -> u8 {
        MAX + 1
    }

    /// Returns all the base cell.
    ///
    /// # Example
    ///
    /// ```
    /// use h3o::BaseCell;
    ///
    /// let cells = BaseCell::iter().collect::<Vec<_>>();
    /// ```
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..Self::count()).map(Self::new_unchecked)
    }

    /// Returns whether or not the tested face is a cw offset face on this cell.
    pub(crate) fn is_cw_offset(self, face: Face) -> bool {
        self.metadata()
            .cw_offset_pent
            .map(|(offset1, offset2)| offset1 == face || offset2 == face)
            .unwrap_or_default()
    }

    /// Returns the number of 60° ccw rotations for that base cell's coordinate
    /// system.
    pub(crate) fn rotation_count(self, face: Face) -> u8 {
        let shift = usize::from(face) * 3;
        let rotation =
            BASE_CELL_ROTATIONS[usize::from(self.0)] >> shift & 0b111;

        debug_assert_ne!(rotation, 0b111, "no cell {self} on face {face:?}");

        rotation as u8
    }

    /// Returns true if the base cell is a pentagon where all neighbors are
    /// oriented towards it.
    pub(crate) const fn is_polar_pentagon(self) -> bool {
        self.0 == 4 || self.0 == 117
    }

    /// Returns the direction-to-face mapping of this pentagonal cell.
    ///
    /// Note that faces are in directional order, starting at J.
    pub(crate) const fn pentagon_direction_faces(
        self,
    ) -> [Face; NUM_PENT_VERTS as usize] {
        debug_assert!(self.is_pentagon(), "not a pentagon");

        let mask = (1_u128 << self.0) - 1;
        let index = (BASE_PENTAGONS & mask).count_ones();
        PENTAGON_DIRECTION_FACES[index as usize]
    }

    /// Returns the neighboring base cell in the given direction.
    ///
    /// Return `None` for pentagonal base cells in the K axe.
    pub(crate) fn neighbor(self, direction: Direction) -> Option<Self> {
        let value = NEIGHBORS[usize::from(self)][usize::from(direction)];

        Self::try_from(value).ok()
    }

    /// Returns the neighboring base cell rotation in the given direction.
    ///
    /// Must be called on a valid direction for the current cell.
    pub(crate) fn neighbor_rotation(self, direction: Direction) -> u8 {
        let base = usize::from(self);
        let to = usize::from(direction);

        debug_assert_ne!(NEIGHBOR_60CCW_ROTS[base][to], 0xff);
        NEIGHBOR_60CCW_ROTS[base][to]
    }

    /// Returns the direction from the origin base cell to the neighbor.
    ///
    /// Returns `None` if the base cells are not neighbors.
    pub(crate) fn direction(self, neighbor: Self) -> Option<Direction> {
        NEIGHBORS[usize::from(self)]
            .iter()
            .position(|&cell| u8::from(neighbor) == cell)
            .map(|dir| {
                // Cast safe thx to bounds.
                #[allow(clippy::cast_possible_truncation)]
                // SAFETY: `i` is bounded in [0; 6].
                Direction::new_unchecked(dir as u8)
            })
    }

    /// Returns base cell metadata.
    fn metadata(self) -> &'static Metadata {
        &METADATA[usize::from(self.0)]
    }
}

impl TryFrom<u8> for BaseCell {
    type Error = error::InvalidBaseCell;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MAX {
            return Err(Self::Error::new(value, "out of range"));
        }
        Ok(Self(value))
    }
}

impl From<BaseCell> for u8 {
    fn from(value: BaseCell) -> Self {
        value.0
    }
}

impl From<BaseCell> for usize {
    fn from(value: BaseCell) -> Self {
        Self::from(value.0)
    }
}

impl From<BaseCell> for FaceIJK {
    fn from(value: BaseCell) -> Self {
        let metadata = value.metadata();
        Self {
            face: metadata.home,
            coord: metadata.coord,
        }
    }
}

impl fmt::Display for BaseCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// -----------------------------------------------------------------------------

/// Base cell lookup table for each face.
///
/// To reduce the footprints of the lookup table, we use a bitset where the
/// rotation is encoded on 3-bit, where `111` means no rotation for this face.
#[allow(clippy::unusual_byte_groupings)] // Grouping by 3 is more explicit here.
const BASE_CELL_ROTATIONS: [u64; BaseCell::count() as usize] = [
    // face 19  18  17  16  15  14  13  12  11  10  9   8   7   6   5   4   3   2   1   0
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_001_000_101,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_000_101_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000_101,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_001_000_101_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_100_011_010_001_000,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_000_101,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000_101_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_101_111_111_001_000,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_101_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_101,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_001_000_101_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_000_101_111_111,
    0b1111_111_111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_001_000_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_000_101_111_111_001,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_101_111_111_001_000,
    0b1111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_111_011_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111,
    0b1111_111_111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_011_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_101_111_111_111_000,
    0b1111_111_111_111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_011_111,
    0b1111_111_111_111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_001_000,
    0b1111_111_111_111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_011_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000_101_111_111,
    0b1111_111_111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_000_101_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_101_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000,
    0b1111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_101_111_111_001,
    0b1111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_111_011,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_011_101_111_111_111_000,
    0b1111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_111_011_111_111,
    0b1111_111_111_111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_011_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_001_000_111_111,
    // face 19  18  17  16  15  14  13  12  11  10  9   8   7   6   5   4   3   2   1   0
    0b1111_111_111_111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_001,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_001_000_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_101_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111,
    0b1111_111_111_111_011_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_011_111_111_111,
    0b1111_111_111_111_111_111_011_111_111_111_111_111_111_111_111_000_111_111_111_111_011,
    0b1111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_011_000_111_111_111_001,
    0b1111_111_111_111_111_111_011_111_111_111_011_111_111_111_111_000_111_111_111_111_011,
    0b1111_111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111_111,
    0b1111_111_111_111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111,
    0b1111_111_111_111_111_011_111_111_111_111_000_111_111_111_011_001_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_001_000_111_111_111,
    0b1111_111_111_111_111_011_111_111_111_111_000_111_111_111_011_111_111_111_111_111_111,
    0b1111_111_111_111_011_111_111_111_111_000_111_111_111_011_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_111,
    0b1111_111_111_111_111_111_111_011_111_111_111_111_000_111_111_111_111_011_111_111_111,
    0b1111_111_111_111_000_001_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_011_111_111_111,
    0b1111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_011_111_111_111_111,
    0b1111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_011_111_111_111_111_111,
    0b1111_111_111_111_111_111_011_111_111_111_111_111_111_111_111_000_111_111_111_111_111,
    0b1111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_011_111_111_111_011_111_111_111_111_000_111_111_111_111_111,
    0b1111_111_111_011_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111_111_111,
    0b1111_111_111_000_001_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111_111,
    0b1111_111_111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111,
    0b1111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_011_111_111_111_111,
    0b1111_111_111_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111,
    // face 19  18  17  16  15  14  13  12  11  10  9   8   7   6   5   4   3   2   1   0
    0b1111_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_101_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_101_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111,
    0b1111_001_111_111_111_000_011_111_111_111_011_111_111_111_111_000_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111_111_111,
    0b1111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_011_111_111_111_111_111,
    0b1111_111_111_111_111_111_111_011_111_111_111_000_111_111_111_111_111_111_111_111_111,
    0b1111_011_111_111_111_111_000_111_111_111_111_111_111_111_111_011_111_111_111_111_111,
    0b1111_111_111_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_011_111_111_111_111_000_111_111_111_011_111_111_111_111_111_111_111_111,
    0b1111_111_111_101_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_001_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111_111_111_111,
    0b1111_001_111_111_101_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_011_111_111_111_111_000_111_111_111_011_011_111_111_111_111_111_111_111_111,
    0b1111_111_000_001_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111_111_111,
    0b1111_111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111,
    0b1111_111_111_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_111_111_111_101_011_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_011_111_111_111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111,
    0b1111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_101_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_011_111_111_111_111_000_111_111_111_011_111_111_111_111_111_111_111_111_111,
    0b1111_111_101_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_111_101_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_001_111_111_111_011_011_111_111_111_000_111_111_111_111_111_111_111_111_111,
    0b1111_001_111_111_111_000_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_001_111_111_101_000_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_101_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_001_111_111_101_011_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_101_000_111_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_111_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_001_010_011_100_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_001_111_111_101_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_101_000_001_111_111_111_011_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    0b1111_101_000_001_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111_111,
    // face 19  18  17  16  15  14  13  12  11  10  9   8   7   6   5   4   3   2   1   0
];

// -----------------------------------------------------------------------------

/// Base cell associated metadata.
struct Metadata {
    /// Home face.
    home: Face,
    /// `IJK` coordinates on the home face.
    coord: CoordIJK,
    /// For pentagon only, the two clockwise offset rotation adjacent faces (if any).
    cw_offset_pent: Option<(Face, Face)>,
}

macro_rules! metadata {
    ($home:literal, [$i:literal, $j:literal, $k: literal]) => {
        Metadata {
            home: Face::new_unchecked($home),
            coord: CoordIJK::new($i, $j, $k),
            cw_offset_pent: None,
        }
    };
    ($home:literal, [$i:literal, $j:literal, $k: literal], ($offset1:literal, $offset2:literal)) => {
        Metadata {
            home: Face::new_unchecked($home),
            coord: CoordIJK::new($i, $j, $k),
            cw_offset_pent: Some((
                Face::new_unchecked($offset1),
                Face::new_unchecked($offset2),
            )),
        }
    };
}

/// Base cell metadata table.
#[rustfmt::skip]
const METADATA: [Metadata; 122] = [
    metadata!(1,  [1, 0, 0]),
    metadata!(2,  [1, 1, 0]),
    metadata!(1,  [0, 0, 0]),
    metadata!(2,  [1, 0, 0]),
    metadata!(0,  [2, 0, 0]),
    metadata!(1,  [1, 1, 0]),
    metadata!(1,  [0, 0, 1]),
    metadata!(2,  [0, 0, 0]),
    metadata!(0,  [1, 0, 0]),
    metadata!(2,  [0, 1, 0]),
    metadata!(1,  [0, 1, 0]),
    metadata!(1,  [0, 1, 1]),
    metadata!(3,  [1, 0, 0]),
    metadata!(3,  [1, 1, 0]),
    metadata!(11, [2, 0, 0], (2, 6)),
    metadata!(4,  [1, 0, 0]),
    metadata!(0,  [0, 0, 0]),
    metadata!(6,  [0, 1, 0]),
    metadata!(0,  [0, 0, 1]),
    metadata!(2,  [0, 1, 1]),
    metadata!(7,  [0, 0, 1]),
    metadata!(2,  [0, 0, 1]),
    metadata!(0,  [1, 1, 0]),
    metadata!(6,  [0, 0, 1]),
    metadata!(10, [2, 0, 0], (1, 5)),
    metadata!(6,  [0, 0, 0]),
    metadata!(3,  [0, 0, 0]),
    metadata!(11, [1, 0, 0]),
    metadata!(4,  [1, 1, 0]),
    metadata!(3,  [0, 1, 0]),
    metadata!(0,  [0, 1, 1]),
    metadata!(4,  [0, 0, 0]),
    metadata!(5,  [0, 1, 0]),
    metadata!(0,  [0, 1, 0]),
    metadata!(7,  [0, 1, 0]),
    metadata!(11, [1, 1, 0]),
    metadata!(7,  [0, 0, 0]),
    metadata!(10, [1, 0, 0]),
    metadata!(12, [2, 0, 0], (3, 7)),
    metadata!(6,  [1, 0, 1]),
    metadata!(7,  [1, 0, 1]),
    metadata!(4,  [0, 0, 1]),
    metadata!(3,  [0, 0, 1]),
    metadata!(3,  [0, 1, 1]),
    metadata!(4,  [0, 1, 0]),
    metadata!(6,  [1, 0, 0]),
    metadata!(11, [0, 0, 0]),
    metadata!(8,  [0, 0, 1]),
    metadata!(5,  [0, 0, 1]),
    metadata!(14, [2, 0, 0], (0, 9)),
    metadata!(5,  [0, 0, 0]),
    metadata!(12, [1, 0, 0]),
    metadata!(10, [1, 1, 0]),
    metadata!(4,  [0, 1, 1]),
    metadata!(12, [1, 1, 0]),
    metadata!(7,  [1, 0, 0]),
    metadata!(11, [0, 1, 0]),
    metadata!(10, [0, 0, 0]),
    metadata!(13, [2, 0, 0], (4, 8)),
    metadata!(10, [0, 0, 1]),
    metadata!(11, [0, 0, 1]),
    metadata!(9,  [0, 1, 0]),
    metadata!(8,  [0, 1, 0]),
    metadata!(6,  [2, 0, 0], (11, 15)),
    metadata!(8,  [0, 0, 0]),
    metadata!(9,  [0, 0, 1]),
    metadata!(14, [1, 0, 0]),
    metadata!(5,  [1, 0, 1]),
    metadata!(16, [0, 1, 1]),
    metadata!(8,  [1, 0, 1]),
    metadata!(5,  [1, 0, 0]),
    metadata!(12, [0, 0, 0]),
    metadata!(7,  [2, 0, 0], (12, 16)),
    metadata!(12, [0, 1, 0]),
    metadata!(10, [0, 1, 0]),
    metadata!(9,  [0, 0, 0]),
    metadata!(13, [1, 0, 0]),
    metadata!(16, [0, 0, 1]),
    metadata!(15, [0, 1, 1]),
    metadata!(15, [0, 1, 0]),
    metadata!(16, [0, 1, 0]),
    metadata!(14, [1, 1, 0]),
    metadata!(13, [1, 1, 0]),
    metadata!(5,  [2, 0, 0], (10, 19)),
    metadata!(8,  [1, 0, 0]),
    metadata!(14, [0, 0, 0]),
    metadata!(9,  [1, 0, 1]),
    metadata!(14, [0, 0, 1]),
    metadata!(17, [0, 0, 1]),
    metadata!(12, [0, 0, 1]),
    metadata!(16, [0, 0, 0]),
    metadata!(17, [0, 1, 1]),
    metadata!(15, [0, 0, 1]),
    metadata!(16, [1, 0, 1]),
    metadata!(9,  [1, 0, 0]),
    metadata!(15, [0, 0, 0]),
    metadata!(13, [0, 0, 0]),
    metadata!(8,  [2, 0, 0], (13, 17)),
    metadata!(13, [0, 1, 0]),
    metadata!(17, [1, 0, 1]),
    metadata!(19, [0, 1, 0]),
    metadata!(14, [0, 1, 0]),
    metadata!(19, [0, 1, 1]),
    metadata!(17, [0, 1, 0]),
    metadata!(13, [0, 0, 1]),
    metadata!(17, [0, 0, 0]),
    metadata!(16, [1, 0, 0]),
    metadata!(9,  [2, 0, 0], (14, 18)),
    metadata!(15, [1, 0, 1]),
    metadata!(15, [1, 0, 0]),
    metadata!(18, [0, 1, 1]),
    metadata!(18, [0, 0, 1]),
    metadata!(19, [0, 0, 1]),
    metadata!(17, [1, 0, 0]),
    metadata!(19, [0, 0, 0]),
    metadata!(18, [0, 1, 0]),
    metadata!(18, [1, 0, 1]),
    metadata!(19, [2, 0, 0]),
    metadata!(19, [1, 0, 0]),
    metadata!(18, [0, 0, 0]),
    metadata!(19, [1, 0, 1]),
    metadata!(18, [1, 0, 0]),
];

// -----------------------------------------------------------------------------

macro_rules! faces {
    [$($face:literal),+] => {
        [$(Face::new_unchecked($face),)*]
    }
}

/// Table of direction-to-face mapping for each pentagon.
///
/// Note that faces are in directional order, starting at J.
#[rustfmt::skip]
const PENTAGON_DIRECTION_FACES: [[Face; NUM_PENT_VERTS as usize]; NUM_PENTAGONS as usize] = [
    faces!( 4,  0,  2,  1,  3),
    faces!( 6, 11,  2,  7,  1),
    faces!( 5, 10,  1,  6,  0),
    faces!( 7, 12,  3,  8,  2),
    faces!( 9, 14,  0,  5,  4),
    faces!( 8, 13,  4,  9,  3),
    faces!(11,  6, 15, 10, 16),
    faces!(12,  7, 16, 11, 17),
    faces!(10,  5, 19, 14, 15),
    faces!(13,  8, 17, 12, 18),
    faces!(14,  9, 18, 13, 19),
    faces!(15, 19, 17, 18, 16),
];

// -----------------------------------------------------------------------------

/// Neighboring base cell in each `IJK` direction.
///
/// For each base cell, for each direction, the neighboring base cell is given.
///
/// `0xff` indicates there is no neighbor in that direction.
#[rustfmt::skip]
const NEIGHBORS: [[u8; 7]; BaseCell::count() as usize] = [
    [  0,     1,   5,   2,   4,   3,   8],
    [  1,     7,   6,   9,   0,   3,   2],
    [  2,     6,  10,  11,   0,   1,   5],
    [  3,    13,   1,   7,   4,  12,   0],
    [  4,  0xff,  15,   8,   3,   0,  12],
    [  5,     2,  18,  10,   8,   0,  16],
    [  6,    14,  11,  17,   1,   9,   2],
    [  7,    21,   9,  19,   3,  13,   1],
    [  8,     5,  22,  16,   4,   0,  15],
    [  9,    19,  14,  20,   1,   7,   6],
    [ 10,    11,  24,  23,   5,   2,  18],
    [ 11,    17,  23,  25,   2,   6,  10],
    [ 12,    28,  13,  26,   4,  15,   3],
    [ 13,    26,  21,  29,   3,  12,   7],
    [ 14,  0xff,  17,  27,   9,  20,   6],
    [ 15,    22,  28,  31,   4,   8,  12],
    [ 16,    18,  33,  30,   8,   5,  22],
    [ 17,    11,  14,   6,  35,  25,  27],
    [ 18,    24,  30,  32,   5,  10,  16],
    [ 19,    34,  20,  36,   7,  21,   9],
    [ 20,    14,  19,   9,  40,  27,  36],
    [ 21,    38,  19,  34,  13,  29,   7],
    [ 22,    16,  41,  33,  15,   8,  31],
    [ 23,    24,  11,  10,  39,  37,  25],
    [ 24,  0xff,  32,  37,  10,  23,  18],
    [ 25,    23,  17,  11,  45,  39,  35],
    [ 26,    42,  29,  43,  12,  28,  13],
    [ 27,    40,  35,  46,  14,  20,  17],
    [ 28,    31,  42,  44,  12,  15,  26],
    [ 29,    43,  38,  47,  13,  26,  21],
    [ 30,    32,  48,  50,  16,  18,  33],
    [ 31,    41,  44,  53,  15,  22,  28],
    [ 32,    30,  24,  18,  52,  50,  37],
    [ 33,    30,  49,  48,  22,  16,  41],
    [ 34,    19,  38,  21,  54,  36,  51],
    [ 35,    46,  45,  56,  17,  27,  25],
    [ 36,    20,  34,  19,  55,  40,  54],
    [ 37,    39,  52,  57,  24,  23,  32],
    [ 38,  0xff,  34,  51,  29,  47,  21],
    [ 39,    37,  25,  23,  59,  57,  45],
    [ 40,    27,  36,  20,  60,  46,  55],
    [ 41,    49,  53,  61,  22,  33,  31],
    [ 42,    58,  43,  62,  28,  44,  26],
    [ 43,    62,  47,  64,  26,  42,  29],
    [ 44,    53,  58,  65,  28,  31,  42],
    [ 45,    39,  35,  25,  63,  59,  56],
    [ 46,    60,  56,  68,  27,  40,  35],
    [ 47,    38,  43,  29,  69,  51,  64],
    [ 48,    49,  30,  33,  67,  66,  50],
    [ 49,  0xff,  61,  66,  33,  48,  41],
    [ 50,    48,  32,  30,  70,  67,  52],
    [ 51,    69,  54,  71,  38,  47,  34],
    [ 52,    57,  70,  74,  32,  37,  50],
    [ 53,    61,  65,  75,  31,  41,  44],
    [ 54,    71,  55,  73,  34,  51,  36],
    [ 55,    40,  54,  36,  72,  60,  73],
    [ 56,    68,  63,  77,  35,  46,  45],
    [ 57,    59,  74,  78,  37,  39,  52],
    [ 58,  0xff,  62,  76,  44,  65,  42],
    [ 59,    63,  78,  79,  39,  45,  57],
    [ 60,    72,  68,  80,  40,  55,  46],
    [ 61,    53,  49,  41,  81,  75,  66],
    [ 62,    43,  58,  42,  82,  64,  76],
    [ 63,  0xff,  56,  45,  79,  59,  77],
    [ 64,    47,  62,  43,  84,  69,  82],
    [ 65,    58,  53,  44,  86,  76,  75],
    [ 66,    67,  81,  85,  49,  48,  61],
    [ 67,    66,  50,  48,  87,  85,  70],
    [ 68,    56,  60,  46,  90,  77,  80],
    [ 69,    51,  64,  47,  89,  71,  84],
    [ 70,    67,  52,  50,  83,  87,  74],
    [ 71,    89,  73,  91,  51,  69,  54],
    [ 72,  0xff,  73,  55,  80,  60,  88],
    [ 73,    91,  72,  88,  54,  71,  55],
    [ 74,    78,  83,  92,  52,  57,  70],
    [ 75,    65,  61,  53,  94,  86,  81],
    [ 76,    86,  82,  96,  58,  65,  62],
    [ 77,    63,  68,  56,  93,  79,  90],
    [ 78,    74,  59,  57,  95,  92,  79],
    [ 79,    78,  63,  59,  93,  95,  77],
    [ 80,    68,  72,  60,  99,  90,  88],
    [ 81,    85,  94, 101,  61,  66,  75],
    [ 82,    96,  84,  98,  62,  76,  64],
    [ 83,  0xff,  74,  70, 100,  87,  92],
    [ 84,    69,  82,  64,  97,  89,  98],
    [ 85,    87, 101, 102,  66,  67,  81],
    [ 86,    76,  75,  65, 104,  96,  94],
    [ 87,    83, 102, 100,  67,  70,  85],
    [ 88,    72,  91,  73,  99,  80, 105],
    [ 89,    97,  91, 103,  69,  84,  71],
    [ 90,    77,  80,  68, 106,  93,  99],
    [ 91,    73,  89,  71, 105,  88, 103],
    [ 92,    83,  78,  74, 108, 100,  95],
    [ 93,    79,  90,  77, 109,  95, 106],
    [ 94,    86,  81,  75, 107, 104, 101],
    [ 95,    92,  79,  78, 109, 108,  93],
    [ 96,   104,  98, 110,  76,  86,  82],
    [ 97,  0xff,  98,  84, 103,  89, 111],
    [ 98,   110,  97, 111,  82,  96,  84],
    [ 99,    80, 105,  88, 106,  90, 113],
    [100,   102,  83,  87, 108, 114,  92],
    [101,   102, 107, 112,  81,  85,  94],
    [102,   101,  87,  85, 114, 112, 100],
    [103,    91,  97,  89, 116, 105, 111],
    [104,   107, 110, 115,  86,  94,  96],
    [105,    88, 103,  91, 113,  99, 116],
    [106,    93,  99,  90, 117, 109, 113],
    [107,  0xff, 101,  94, 115, 104, 112],
    [108,   100,  95,  92, 118, 114, 109],
    [109,   108,  93,  95, 117, 118, 106],
    [110,    98, 104,  96, 119, 111, 115],
    [111,    97, 110,  98, 116, 103, 119],
    [112,   107, 102, 101, 120, 115, 114],
    [113,    99, 116, 105, 117, 106, 121],
    [114,   112, 100, 102, 118, 120, 108],
    [115,   110, 107, 104, 120, 119, 112],
    [116,   103, 119, 111, 113, 105, 121],
    [117,  0xff, 109, 118, 113, 121, 106],
    [118,   120, 108, 114, 117, 121, 109],
    [119,   111, 115, 110, 121, 116, 120],
    [120,   115, 114, 112, 121, 119, 118],
    [121,   116, 120, 119, 117, 113, 118],
];

// -----------------------------------------------------------------------------

/// Neighboring base cell rotations in each `IJK` direction.
///
/// For each base cell, for each direction, the number of 60 degree
/// CCW rotations to the coordinate system of the neighbor is given.
///
/// `0xff` indicates there is no neighbor in that direction.
#[rustfmt::skip]
const NEIGHBOR_60CCW_ROTS: [[u8; 7]; BaseCell::count() as usize] = [
    [0,    5, 0, 0, 1, 5, 1],
    [0,    0, 1, 0, 1, 0, 1],
    [0,    0, 0, 0, 0, 5, 0],
    [0,    5, 0, 0, 2, 5, 1],
    [0, 0xff, 1, 0, 3, 4, 2],
    [0,    0, 1, 0, 1, 0, 1],
    [0,    0, 0, 3, 5, 5, 0],
    [0,    0, 0, 0, 0, 5, 0],
    [0,    5, 0, 0, 0, 5, 1],
    [0,    0, 1, 3, 0, 0, 1],
    [0,    0, 1, 3, 0, 0, 1],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    5, 0, 0, 3, 5, 1],
    [0,    0, 1, 0, 1, 0, 1],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    5, 0, 0, 4, 5, 1],
    [0,    0, 0, 0, 0, 5, 0],
    [0,    3, 3, 3, 3, 0, 3],
    [0,    0, 0, 3, 5, 5, 0],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    3, 3, 3, 0, 3, 0],
    [0,    0, 0, 3, 5, 5, 0],
    [0,    0, 1, 0, 1, 0, 1],
    [0,    3, 3, 3, 0, 3, 0],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    0, 0, 3, 0, 0, 3],
    [0,    0, 0, 0, 0, 5, 0],
    [0,    3, 0, 0, 0, 3, 3],
    [0,    0, 1, 0, 1, 0, 1],
    [0,    0, 1, 3, 0, 0, 1],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 0, 0, 0, 5, 0],
    [0,    3, 3, 3, 3, 0, 3],
    [0,    0, 1, 3, 0, 0, 1],
    [0,    3, 3, 3, 3, 0, 3],
    [0,    0, 3, 0, 3, 0, 3],
    [0,    0, 0, 3, 0, 0, 3],
    [0,    3, 0, 0, 0, 3, 3],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    3, 0, 0, 3, 3, 0],
    [0,    3, 0, 0, 3, 3, 0],
    [0,    0, 0, 3, 5, 5, 0],
    [0,    0, 0, 3, 5, 5, 0],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 1, 3, 0, 0, 1],
    [0,    0, 3, 0, 0, 3, 3],
    [0,    0, 0, 3, 0, 3, 0],
    [0,    3, 3, 3, 0, 3, 0],
    [0,    3, 3, 3, 0, 3, 0],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    0, 0, 3, 0, 0, 3],
    [0,    3, 0, 0, 0, 3, 3],
    [0,    0, 3, 0, 3, 0, 3],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 3, 0, 3, 0, 3],
    [0,    0, 3, 0, 0, 3, 3],
    [0,    3, 3, 3, 0, 0, 3],
    [0,    0, 0, 3, 0, 3, 0],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    3, 3, 3, 3, 3, 0],
    [0,    3, 3, 3, 3, 3, 0],
    [0,    3, 3, 3, 3, 0, 3],
    [0,    3, 3, 3, 3, 0, 3],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    0, 0, 3, 0, 0, 3],
    [0,    3, 3, 3, 0, 3, 0],
    [0,    3, 0, 0, 0, 3, 3],
    [0,    3, 0, 0, 3, 3, 0],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    3, 0, 0, 3, 3, 0],
    [0,    0, 3, 0, 0, 3, 3],
    [0,    0, 0, 3, 0, 3, 0],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    3, 3, 3, 0, 0, 3],
    [0,    3, 3, 3, 0, 0, 3],
    [0,    0, 0, 3, 0, 0, 3],
    [0,    3, 0, 0, 0, 3, 3],
    [0,    0, 0, 3, 0, 5, 0],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 1, 3, 1, 0, 1],
    [0,    0, 1, 3, 1, 0, 1],
    [0,    0, 3, 0, 3, 0, 3],
    [0,    0, 3, 0, 3, 0, 3],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    0, 3, 0, 0, 3, 3],
    [0,    0, 0, 3, 0, 3, 0],
    [0,    3, 0, 0, 3, 3, 0],
    [0,    3, 3, 3, 3, 3, 0],
    [0,    0, 0, 3, 0, 5, 0],
    [0,    3, 3, 3, 3, 3, 0],
    [0,    0, 0, 0, 0, 0, 1],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 0, 3, 0, 5, 0],
    [0,    5, 0, 0, 5, 5, 0],
    [0,    0, 3, 0, 0, 3, 3],
    [0,    0, 0, 0, 0, 0, 1],
    [0,    0, 0, 3, 0, 3, 0],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    3, 3, 3, 0, 0, 3],
    [0,    5, 0, 0, 5, 5, 0],
    [0,    0, 1, 3, 1, 0, 1],
    [0,    3, 3, 3, 0, 0, 3],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 1, 3, 1, 0, 1],
    [0,    3, 3, 3, 3, 3, 0],
    [0,    0, 0, 0, 0, 0, 1],
    [0,    0, 1, 0, 3, 5, 1],
    [0, 0xff, 3, 0, 5, 2, 0],
    [0,    5, 0, 0, 5, 5, 0],
    [0,    0, 1, 0, 4, 5, 1],
    [0,    3, 3, 3, 0, 0, 0],
    [0,    0, 0, 3, 0, 5, 0],
    [0,    0, 0, 3, 0, 5, 0],
    [0,    0, 1, 0, 2, 5, 1],
    [0,    0, 0, 0, 0, 0, 1],
    [0,    0, 1, 3, 1, 0, 1],
    [0,    5, 0, 0, 5, 5, 0],
    [0, 0xff, 1, 0, 3, 4, 2],
    [0,    0, 1, 0, 0, 5, 1],
    [0,    0, 0, 0, 0, 0, 1],
    [0,    5, 0, 0, 5, 5, 0],
    [0,    0, 1, 0, 1, 5, 1],
];
