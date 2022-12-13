use crate::{
    coord::{CoordIJK, FaceIJK},
    error, Face, NUM_PENTAGONS, NUM_PENT_VERTS,
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