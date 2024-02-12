map(
    select(.event != null)
    | select(.event != "started")
    | select(.type == "test")
  )
