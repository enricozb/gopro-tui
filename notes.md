feature ideas:
  - add importing functionality
  - multithread timezone estimation

usage flow:
  1. invoke cli
    - `import <input-dir> <output-dir>`
  2. input inference
    - detect all MP4 files
      - detect all associated files
    - infer date and timezone for each
  3. output inference
    - detect all output directories
  4. user categorizes each date into an output dir

cli args:
  - input directory
  - output directory

inference state:
  - discovering and associating files
  - inferring timestamps
  - discovering output directories

ui state:
  - inference state
    - discovering inputs
    - inferring timestamps
    - discovering outputs
    - done

  - selected region
    - each region has a specific set of keybinding handlers
      - outputs section should have a keybinding for `new` output folders
      - sessions section should have a keybinding for modifying the `destination` folder

  - files for which a timestamp could not be inferred

resources:
  - [so post](https://stackoverflow.com/questions/16086962/how-to-get-a-time-zone-from-a-location-using-latitude-and-longitude-coordinates)
  - [exiftool to get lat/long](https://www.trekview.org/blog/2020/metadata-exif-xmp-360-video-files-gopro-gpmd/)
  - [extracting gopro telemetry with ffmpeg](https://lucaselbert.medium.com/extracting-gopro-gps-and-other-telemetry-data-fadf97ed1834)
  - [golang gopro metadata parser](https://github.com/stilldavid/gopro-utils)
  - [gopro metadata format spec](https://github.com/gopro/gpmf-parser)
