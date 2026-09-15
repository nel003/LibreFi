import Admin from "./routes/admin";
import User from "./routes/user"
import { Route, Switch } from "wouter";

function App() {
  return (
    <>
      <Switch>
        <Route path="/">
          <User />
        </Route>
        <Route path="/admin" nest>
          <Admin />
        </Route>
      </Switch>
    </>
  )
}

export default App
