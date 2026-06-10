import { useState } from "react"
import DatasourceForm from "./data-forms/datasource-form"
import JobForm from "./data-forms/job-form"
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

export default function FormInfo({
  onRefresh,
  onJobCreated,
}: {
  onRefresh?: () => void
  onJobCreated?: (datasourceId: string) => void
}) {
  const [toggleCreate, setCreate] = useState<string>("datasource")

  return (
    <Card
        size="sm"
        className="h-full gap-0 bg-white shadow-sm transition-shadow hover:shadow-md"
      >
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
              <Button
                disabled={toggleCreate === "datasource"}
                className={cn(
                  "rounded-t-sm! w-50/100 shadow-sm transition-all active:scale-[0.98] disabled:opacity-100",
                  toggleCreate === "datasource"
                    ? "bg-[#8828ad] text-white hover:bg-[#8828ad]/90 border-1 border-[#8e25b6]! "
                    : "bg-gray-100 text-gray-500 hover:bg-gray-200",
                )}
                onClick={() => setCreate("datasource")}
              >
                Datasource
              </Button>
              <Button
                disabled={toggleCreate === "job"}
                className={cn(
                  "rounded-t-sm! w-50/100 shadow-sm transition-all active:scale-[0.98] disabled:opacity-100",
                  toggleCreate === "job"
                    ? "bg-[#8828ad] text-white hover:bg-[#8828ad]/90 border-1 border-[#8e25b6]!"
                    : "bg-gray-100 text-gray-500 hover:bg-gray-200 ",
                )}
                onClick={() => setCreate("job")}
              >
                Job
              </Button>
          </CardTitle>
        </CardHeader>
        <CardContent className="flex items-baseline justify-center gap-x-4 gap-y-2 text-sm bg-[#f8eefc] h-full border-1 border-[#8e25b6]! mx-4 rounded-b-sm!">
          {
            toggleCreate === "datasource"
              ? <DatasourceForm onRefresh={onRefresh}/>
              : <JobForm onJobCreated={onJobCreated}/>
          }
        </CardContent>
      </Card>
  )
}
