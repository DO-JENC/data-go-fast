import DatasourceCard from "@/components/datasources/datasource-card"
import TreePanel from "@/components/datasources/tree-panel"
import { useDatasources } from "@/hooks/use-datasources"
import { useState } from "react"
import type { Datasource } from "@/types/datasource"
import type { Job } from "@/types/job"

export default function Datasources() {
  const { removeDatasource } = useDatasources()
  const [selected, setSelected] = useState<{
    item: Datasource | Job | string
    type: "datasource" | "job" | "form"
  } | null>(null)

  return (
    <div className="flex min-h-0 flex-1 gap-6 p-6 text-left">
      <aside className="w-[280px] shrink-0">
        <TreePanel onSelect={(item, type) => setSelected({ item, type })} />
      </aside>
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
        <div className="flex flex-row items-center justify-between">
          <h1 className="mb-2 mt-0 text-2xl font-medium">
            {selected
              ? selected.type === "datasource"
                ? `Datasources`
              : selected.type === "job"
                ? `Job`
                : `Add an entity`
              : ``
            }
          </h1>
        </div>
        <DatasourceCard
          item={selected?.item ?? null}
          type={selected?.type ?? null}
          onDelete={
            selected?.type === "datasource" ? removeDatasource : undefined // TODO : CHECK THAT
          }
        />
      </div>
    </div>
  )
}
