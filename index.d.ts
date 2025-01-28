/* auto-generated by NAPI-RS */
/* eslint-disable */
export declare class DicomFile {
  constructor()
  check(path: string): DicomFileMeta
  open(path: string): string | null
  dump(): void
  getPatientName(): string | null
  getElements(): DicomElements
  saveRawPixelData(path: string): string
  /**
   * Close the DICOM file to free resources
   */
  close(): void
}

/** DICOM C-STORE SCP */
export declare class StoreScp {
  constructor(options: StoreScpOptions)
  listen(): Promise<void>
  close(): Promise<void>
  addEventListener(event: Event, handler: ((err: Error | null, arg: EventData) => any)): void
}
export type StoreSCP = StoreScp

/** DICOM C-STORE SCU */
export declare class StoreScu {
  constructor(options: StoreScuOptions)
  addFile(path: string): void
  addFolder(path: string): void
  send(): Promise<unknown>
}
export type StoreSCU = StoreScu

export interface DicomElements {
  /** Storage SOP Class UID */
  sopClassUid?: string
  /** Storage SOP Instance UID */
  sopInstanceUid?: string
  /** Instance Creation Date */
  instanceCreationDate?: string
  /** Instance Creation Time */
  instanceCreationTime?: string
  /** Study Id */
  studyId?: string
  /** Study Date */
  studyDate?: string
  /** Study Time */
  studyTime?: string
  /** Acquisition DateTime */
  acquisitionDateTime?: string
  /** Modality */
  modality?: string
  /** Manufacturer */
  manufacturer?: string
  /** Manufacturer Model Name */
  manufacturerModelName?: string
  /** Study Description */
  studyDescription?: string
  /** Series Description */
  seriesDescription?: string
  /** Patient Name */
  patientName?: string
  /** Patient ID */
  patientId?: string
  /** Patient Birth Date */
  patientBirthDate?: string
  /** Patient Sex */
  patientSex?: string
  /** Image Comments */
  imageComments?: string
  /** Series Number */
  seriesNumber?: string
  /** Instance Number */
  instanceNumber?: string
}

export interface DicomFileMeta {
  /** Storage SOP Class UID */
  sopClassUid: string
  /** Storage SOP Instance UID */
  sopInstanceUid: string
}

export declare const enum Event {
  OnServerStarted = 'OnServerStarted',
  OnError = 'OnError',
  OnConnection = 'OnConnection',
  OnFileStored = 'OnFileStored'
}

export interface EventData {
  message: string
  data?: string
}

export interface ResultObject {
  /** Transfer Syntax UID */
  status: ResultStatus
  message: string
}

export declare const enum ResultStatus {
  Success = 'Success',
  Error = 'Error'
}

export declare function saveRawPixelData(filePath: string, outPath: string): string

export interface StoreScpOptions {
  /** Verbose mode */
  verbose?: boolean
  /** Calling Application Entity title */
  callingAeTitle?: string
  /** Enforce max pdu length */
  strict?: boolean
  /** Only accept native/uncompressed transfer syntaxes */
  uncompressedOnly?: boolean
  /** Accept unknown SOP classes */
  promiscuous?: boolean
  /** Maximum PDU length */
  maxPduLength?: number
  /** Output directory for incoming objects */
  outDir: string
  /** Which port to listen on */
  port: number
}

export interface StoreScuOptions {
  /**
   * socket address to Store SCP,
   * optionally with AE title
   * (example: "STORE-SCP@127.0.0.1:104")
   */
  addr: string
  /** verbose mode */
  verbose?: boolean
  /** the C-STORE message ID */
  messageId?: number
  /** the calling Application Entity title, [default: STORE-SCU] */
  callingAeTitle?: string
  /**
   * the called Application Entity title,
   * overrides AE title in address if present [default: ANY-SCP]
   */
  calledAeTitle?: string
  /** the maximum PDU length accepted by the SCU [default: 16384] */
  maxPduLength?: number
  /** fail if not all DICOM files can be transferred */
  failFirst?: boolean
  /** fail file transfer if it cannot be done without transcoding */
  neverTranscode?: boolean
  /** User Identity username */
  username?: string
  /** User Identity password */
  password?: string
  /** User Identity Kerberos service ticket */
  kerberosServiceTicket?: string
  /** User Identity SAML assertion */
  samlAssertion?: string
  /** User Identity JWT */
  jwt?: string
  /** Dispatch these many service users to send files in parallel */
  concurrency?: number
}
