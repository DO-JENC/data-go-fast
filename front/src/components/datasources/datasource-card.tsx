import { Card } from "@/components/ui/card"
import type { Datasource } from "@/types/datasource"
import type { Job } from "@/types/job"

import DatasourceInfo from "../cards/datasource-info"
import JobInfo from "../cards/job-info"

export default function DatasourceCard({
  item,
  type,
  onDelete,
}: {
  item: Datasource | Job | null
  type: "datasource" | "job" | null
  onDelete?: (id: string) => Promise<boolean>
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
       onDelete={onDelete}/>
    )
  }
  // else case (for the moment only job but TODO : change when form added)
  const job = item as Job
  return(
    <JobInfo
     job={job}/>
  )
}
