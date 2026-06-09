import type { DatasourceType } from "./datasource"

export interface Job {
  job_id: string
  datasource_id: string
  name: string
  pipeline: PipelineOp[]
  status: string
  result_datasource_id: string | null
}

export type PipelineOp = IngestOp | FilterOp

export interface IngestOp {
  op: "ingest"
  type: DatasourceType
  header: boolean
}

export interface FilterOp {
  op: "filter"
  column: string
  operator: string
  value: unknown
}
