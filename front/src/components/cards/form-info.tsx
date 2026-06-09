import { useState } from "react"
import DatasourceForm from "./data-forms/datasource-form"
import JobForm from "./data-forms/job-form"
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card"
import { Button } from "@/components/ui/button"


export default function FormInfo() {
  const [toggleCreate, setCreate] = useState<string>("datasource")

  return (
    <Card
        size="sm"
        className="h-full bg-white shadow-sm transition-shadow hover:shadow-md"
      >
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
              <Button
                disabled={toggleCreate === "datasource"}
                className="rounded-sm! w-50/100 bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
                // size="default"
                onClick={() => setCreate("datasource")}
              >
                Datasource
              </Button>
              <Button
                disabled={toggleCreate === "job"}
                className="rounded-sm! w-50/100 bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
                // size="default"
                onClick={() => setCreate("job")}
              >
                Job
              </Button>
          </CardTitle>
        </CardHeader>
        <CardContent className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
          {
            toggleCreate === "datasource"
              ? <DatasourceForm/>
              : <JobForm/>
          }
        </CardContent>
      </Card>
  )
}