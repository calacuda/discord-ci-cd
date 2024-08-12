use anyhow::{bail, Result};
use crossbeam::channel::{Receiver, Sender};
use docker_command::{command_run::Command, BuildOpt, Launcher, RunOpt, Volume};
use git2::Repository;
use poise::serenity_prelude::futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs::{read_to_string, remove_dir_all},
    spawn,
    task::JoinHandle,
    time::sleep,
};
use url::Url;

pub type PipelineName = String;
pub type RepoName = String;
pub type Pipelines = HashMap<PipelineName, Pipeline>;

pub const CACHE_DIR: &str = &"/tmp/dcicd/";
pub const PIPELINE_FILE: &str = &".dcicd.toml";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Repo {
    pub repo_name: RepoName,
    pub url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Pipeline {
    // pub name: PipelineName,
    pub container: String,
    pub script: Vec<String>,
    // pub script_loc: usize,
    pub artifacts: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BackendState {
    /// a pipeline is running
    RunningPipeline { repo: Repo, pipeline: PipelineName },
    /// available to run pipelines
    Available { repo: Repo },
    /// no repos is "loaded"
    #[default]
    NotConfigured,
}

// #[derive(Debug, Clone)]
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// #[derive(Clone)]
pub enum CiCdCmd {
    Clone(Url),
    RunPipeline {
        pipeline_name: PipelineName,
        // token: String,
        // ctx: ,
        on_complete: Box<dyn Fn(String) + Send + Sync>,
        // on_complete: Context<'a>,
    },
    GetLogs,
}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Pipeline {}

// #[derive(Debug, Clone, Default, PartialEq, Eq)]
// #[derive(Debug, Clone)]
#[derive(Debug)]
pub struct Backend {
    /// tells if the back end is running, available, etc
    pub state: Arc<Mutex<BackendState>>,
    pub repos: HashMap<RepoName, Url>,
    pub jh: JoinHandle<()>,
    // pub input: Receiver<CiCdCmd>,
    pub output: Sender<String>,
    pub logs: Arc<Mutex<String>>,
}

impl Backend {
    // pub fn new(input: Receiver<CiCdCmd>, output: Sender<String>) -> Self {
    pub fn new(output: Sender<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(BackendState::default())),
            repos: HashMap::default(),
            jh: spawn(async { () }),
            // input,
            output,
            logs: Arc::new(Mutex::new(String::default())),
        }
    }

    pub async fn process(&mut self, msg: CiCdCmd) -> Result<()> {
        match msg {
            CiCdCmd::GetLogs => {
                let logs = self.logs.lock().await.clone();
                self.output.send(logs).unwrap();
            }
            // CiCdCmd::RunPipeline(pipeline_name) => {
            CiCdCmd::RunPipeline {
                pipeline_name,
                on_complete,
            } => {
                println!("pre-repo");

                let repo = {
                    match self.state.lock().await.clone() {
                        BackendState::Available { repo } => repo,
                        _ => bail!("backend is not available to run jobs at the moment. pls wait for job to finish."),
                    }
                };

                println!("{repo:?}");

                // load repos pipeline file.
                let mut pipeline_file: PathBuf = PathBuf::from(CACHE_DIR);
                pipeline_file.push(PIPELINE_FILE);

                let Ok(pipelines) = toml::from_str::<Pipelines>(
                    &read_to_string(&pipeline_file)
                        .await
                        .unwrap_or(String::new()),
                ) else {
                    self.output
                        .send(format!("failed to read {PIPELINE_FILE}"))
                        .unwrap();
                    bail!(format!(
                        "failed to read {PIPELINE_FILE}, {:?}",
                        toml::from_str::<Pipelines>(
                            &read_to_string(&pipeline_file)
                                .await
                                .unwrap_or(String::new())
                        )
                    ));
                };

                // find pipline
                let Some(pipeline) = pipelines.get(&pipeline_name).map(|pl| pl.to_owned()) else {
                    self.output
                        .send(format!("unknown pipeline: {pipeline_name}"))
                        .unwrap();
                    bail!(format!("unknown pipeline: {pipeline_name}"));
                };

                // TODO: build docker container
                let launcher = Launcher::new(Command {
                    program: PathBuf::from("/usr/bin/docker"),
                    ..Default::default()
                });
                launcher
                    .build(BuildOpt {
                        build_args: vec![("BASE_IMAGE".into(), pipeline.container)],
                        context: PathBuf::from("/etc/dcicd/docker/"),
                        tag: Some("dcicd".into()),
                        no_cache: true,
                        ..Default::default()
                    })
                    .run()?;

                // TODO: run pipeline
                let state = self.state.clone();
                let logs = self.logs.clone();

                self.jh = spawn(run(state, logs, on_complete, repo, pipeline_name, launcher));
            }
            CiCdCmd::Clone(url) => {
                match *self.state.lock().await {
                    BackendState::Available { repo: _ } => {}
                    _ => bail!("backend is not available to clone at the moment. pls wait for job to finish."),
                }

                // RM storage dir
                let path = Path::new(CACHE_DIR);

                if path.exists() {
                    remove_dir_all(path)
                        .await
                        .expect("Could not remove old socket!");
                }

                // clone repo to storage dir
                if let Err(e) = Repository::clone(&url.to_string(), CACHE_DIR) {
                    self.output
                        .send(format!("failed to clone repo: {}", e))
                        .unwrap();
                    bail!(format!("failed to clone repo: {}", e));
                }
            }
        };

        Ok(())
    }
}

async fn run(
    state: Arc<Mutex<BackendState>>,
    logs: Arc<Mutex<String>>,
    on_complete: Box<dyn Fn(String) + Send + Sync>,
    repo: Repo,
    pipeline_name: String,
    launcher: Launcher,
) {
    {
        let mut s = state.lock().await;
        *s = BackendState::RunningPipeline {
            repo: repo.clone(),
            pipeline: pipeline_name.clone(),
        };
    }

    // mount the git repo as a volume in a custom docker container at:
    // /home/dcicd-runner/repo/. have the docker container run the CiCd pipeline.
    // docker build docker-files/runner/. --build-arg="BASE_IMAGE=rust" -t test-runner
    let res = launcher
        .run(RunOpt {
            image: "dcicd".into(),
            remove: true,
            volumes: vec![Volume {
                src: PathBuf::from(CACHE_DIR),
                dst: PathBuf::from("/home/dcicd-runner/repo/"),
                read_write: true,
                ..Default::default()
            }],
            command: Some(pipeline_name.into()),
            ..Default::default()
        })
        .combine_output()
        .enable_capture()
        .run();

    let mut l = logs.lock().await;

    // if let Err(e) = res.map(|res| {
    //     *l = String::from_utf8_lossy(&res.stdout).to_string();
    //
    //     on_complete(&format!("pipline run completed sucessfully.")).await;
    //     // on_complete();
    // }) {
    //     eprintln!("failed to launch runner. failed with error, {e}");
    //     *l = "failed to launch runner (Docker/Podman). failed with error, {e}".into();
    // }

    match res {
        Ok(res) => {
            *l = String::from_utf8_lossy(&res.stdout).to_string();

            on_complete(format!(
                "pipline run completed sucessfully. use `/logs` to view logs."
            ));
            // on_complete();
        }
        Err(e) => {
            eprintln!("failed to launch runner. failed with error, {e}");
            *l = "failed to launch runner (Docker/Podman). failed with error, {e}".into();
            on_complete("pipline failed! use `/logs to view logs.".into());
        }
    }

    println!("run done");

    {
        let mut s = state.lock().await;
        *s = BackendState::Available { repo };
    }
}

// impl Default for Backend {
//     fn default() -> Self {
//         Self {
//             state: BackendState::default(),
//             repos: HashMap::default(),
//             jh: spawn(awa)
//         }
//     }
// }

// impl Future for Backend {
//     type Output = ();
//
//     fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         cx.waker().wake_by_ref();
//         Poll::Pending
//     }
// }

pub async fn run_backend(input: Receiver<CiCdCmd>, backend: Arc<Mutex<Backend>>) {
    loop {
        sleep(Duration::from_millis(250)).await;
        let mut backend = backend.lock().await;

        if let Ok(msg) = input.try_recv() {
            // println!("got message {msg:?}");
            if let Err(e) = backend.process(msg).await {
                eprintln!("{e}");
            }
        }
    }
}
