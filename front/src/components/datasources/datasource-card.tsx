import { Card } from "@/components/ui/card"
import type { Datasource } from "@/types/datasource"
import type { Job } from "@/types/job"

import DatasourceInfo from "../cards/datasource-info"
import JobInfo from "../cards/job-info"
import FormInfo from "../cards/form-info"

export default function DatasourceCard({
  item,
  type,
  onDelete,
  onRefresh,
  onJobCreated,
}: {
  item: Datasource | Job | string | null
  type: "datasource" | "job" | "form" | null
  onDelete?: (id: string) => Promise<boolean>
  onRefresh?: () => void
  onJobCreated?: (datasourceId: string) => void
}) {
  if (!item || !type) {
    return (
      <Card
        size="sm"
        className="flex h-full items-center justify-center bg-white shadow-sm"
      >
        <p className="text-sm text-muted-foreground">
          Select an item from the tree
        </p>
      </Card>
    )
  }

  if (type === "datasource") {
    return (
      <DatasourceInfo
       item={item as Datasource}
       onDelete={onDelete}
       onRefresh={onRefresh}/>
    )
  }

  if (type === "form") {
    return (
      <FormInfo onRefresh={onRefresh} onJobCreated={onJobCreated}/>
    )
  }

  const job = item as Job
  return(
    <JobInfo job={job}/>
  )
}
