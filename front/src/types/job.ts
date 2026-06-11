import type { DatasourceType } from "./datasource"

export interface Job {
  job_id: string
  datasource_id: string
  name: string
  pipeline: PipelineOp[]
  status: string
  result_datasource_id: string | null
}

export type PipelineOp = IngestOp | FilterOp | AggregateOp | GroupByOp

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

export interface AggregateOp {
  op: "aggregate"
  columns: string[]
  functions: string[]
}

export interface GroupByOp {
  op: "group_by"
  by: string
  aggregate: {
    column: string
    function: string
  }
}
