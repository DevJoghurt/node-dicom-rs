import { DicomFile } from '../index.js';

const file = new DicomFile();

file.open('./__test__/fixtures/test.dcm');

file.saveRawPixelData('./tmp/raw_pixel_data.jpg');

console.log(file.getElements());

file.close();