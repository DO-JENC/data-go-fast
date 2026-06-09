import { useState } from "react"
import DatasourceForm from "./data-forms/datasource-form"
import JobForm from "./data-forms/job-form"


export default function FormInfo() {
  const [toggleCreate, setCreate] = useState<string>("datasource")

  if (toggleCreate == "datasource") {
    return (
      <DatasourceForm/>
    )
  }

  if (toggleCreate == "job") {
    return (
      <JobForm/>
    )
  }

  return null
}