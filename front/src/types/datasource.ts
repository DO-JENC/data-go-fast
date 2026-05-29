export type DatasourceType = "csv" | "json"

export interface Datasource {
  id: string
  s3_id: string
  name: string
  file_type: DatasourceType | null
  size: number
  created_at: string | null
  group_id: string | null
}
