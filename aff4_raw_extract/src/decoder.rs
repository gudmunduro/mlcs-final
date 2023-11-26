use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use flate2::read::ZlibDecoder;
use itertools::Itertools;
use yaml_rust::YamlLoader;
use zip::ZipArchive;

pub struct SegmentLocation {
    pub start: i64,
    pub length: i64,
}

pub fn decode_file(file_name: &str, object_name: &str) -> anyhow::Result<()> {
    let file_name_no_extension = file_name.split(".").next().unwrap();

    let zip_file = File::open(file_name)?;
    let mut archive = ZipArchive::new(zip_file)?;

    let temp_data_file_name = format!("{file_name_no_extension}-temp");
    let output_file_name = format!("{file_name_no_extension}.raw");

    decode_all_aff4_segments(&mut archive, object_name, &temp_data_file_name)?;
    generate_output_file(&mut archive, object_name, &temp_data_file_name, &output_file_name)?;

    fs::remove_file(&temp_data_file_name)?;

    println!("Successfully decoded {file_name}");
    Ok(())
}

fn decode_all_aff4_segments(archive: &mut ZipArchive<File>, object_name: &str, temp_output_file_name: &str) -> anyhow::Result<()> {
    let mut segments = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .filter(|name| name.starts_with(&format!("{object_name}/data")) && !name.ends_with("index"))
        .collect::<Vec<_>>();
    segments.sort();

    let mut temp_data_file = File::create(temp_output_file_name)?;
    for entry in &segments {
        let index_content = {
            let index_file = &mut archive.by_name(&format!("{entry}/index"))?;
            let mut index_content = Vec::new();
            index_file.read_to_end(&mut index_content)?;

            index_content
        };

        // Convert index file into a list of 32 bit indexes
        let indexes = index_content.into_iter()
            .chunks(4)
            .into_iter()
            .map(|c| {
                let c = c.collect::<Vec<u8>>();
                u32::from_le_bytes(c.as_slice().try_into().unwrap())
            })
            .collect::<Vec<u32>>();

        let bevy_file = &mut archive.by_name(entry)?;
        let mut bevy_content = Vec::new();
        bevy_file.read_to_end(&mut bevy_content)?;

        // Decode every chunk for the image and write into temporary file
        for i in 0..indexes.len() {
            let mut res = Vec::new();
            let start = indexes[i] as usize;

            let mut content = if i < indexes.len() - 1 {
                let end = indexes[i + 1] as usize;

                &bevy_content[start..end]
            } else {
                &bevy_content[start..]
            };

            let mut decoded = Vec::new();
            let mut z = ZlibDecoder::new(&mut content);
            z.read_to_end(&mut decoded)?;

            res.extend(decoded);

            temp_data_file.write_all(res.as_slice())?;
        }

        println!("Segment decoded");
    }

    Ok(())
}

fn generate_output_file(archive: &mut ZipArchive<File>, object_name: &str, temp_data_file_name: &str, output_file_name: &str) -> anyhow::Result<()> {
    let mut info_yaml = String::new();
    archive.by_name(&format!("{object_name}/information.yaml"))?.read_to_string(&mut info_yaml)?;
    let docs = YamlLoader::load_from_str(&info_yaml)?;
    let info_doc = &docs[0];

    let segment_locations = info_doc["Runs"].clone().into_iter().map(|e| SegmentLocation {
        start: e["start"].as_i64().unwrap(),
        length: e["length"].as_i64().unwrap(),
    }).collect::<Vec<SegmentLocation>>();

    // Add gaps to file
    let mut temp_data_file = File::open(temp_data_file_name)?;
    let mut output_file = File::create(output_file_name)?;

    let mut file_pointer = 0_u64;
    for seg_loc in &segment_locations {
        let mut dump_bytes = vec![0; seg_loc.length as usize];
        temp_data_file.seek(SeekFrom::Start(file_pointer))?;
        temp_data_file.read_exact(&mut dump_bytes)?;
        file_pointer += seg_loc.length as u64;

        output_file.seek(SeekFrom::Start(seg_loc.start as u64))?;
        output_file.write(&dump_bytes[..])?;
    }

    Ok(())
}