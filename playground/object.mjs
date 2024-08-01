import { DicomFile } from '../index.js';

const file = new DicomFile();

file.open('./tmp/8B1FA77C.dcm');

file.saveRawPixelData('./tmp/raw_pixel_data.txt');

console.log(file.getElements());