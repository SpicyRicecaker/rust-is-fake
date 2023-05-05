import { invoke } from "@tauri-apps/api/tauri";
import { Component, createResource, createSignal, onMount } from 'solid-js';

import { VsRefresh } from 'solid-icons/vs'
import styles from './App.module.css';

const App: Component = () => {
  const [task, { mutate, refetch }] = createResource(getStuffFromNotion)
  const [sel, setSel] = createSignal(false)

  onMount(async () => {
    await invoke('show_window')
  })

  return (
    <div onMouseEnter={() => setSel(true)} onMouseLeave={() => setSel(false)} class={styles.App}>
      <button style={`opacity: ${sel() ? "100%" : "0%"};`} class={styles.button} onClick={refetch}><VsRefresh size={24} color='white' /></button>
      <header class={styles.header}>
        <span class={styles.task}><b>Current Task:</b> {task()}</span>
      </header>
    </div>
  );
};

// question: how does the user add their secret key and database id? 
// it must be editable from the client. from there, it'd be really annoying to
// have to save the secret key and id every time the application is started up,
// so the secret key and database id should be stored into the server
// upon a get item request, the server should be making the call to the notion
// server, not the client. 

// Gets a single item from notion database. May return an error, or there may be
// no tasks in progress, in which case an empty list is received
const getStuffFromNotion = async (): Promise<string> => {
  let res = await invoke('get_in_progress_item') as any
  return res.task
}

export default App;
