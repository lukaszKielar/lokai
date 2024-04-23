use std::str::FromStr;

use leptos::html::{Div, Textarea};
use leptos::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::frontend::components::message::Messages;
use crate::server::api::{get_conversation_messages, AskAssistant};
use crate::{models::Message as MessageModel, MODEL};

#[component]
pub(crate) fn Conversation() -> impl IntoView {
    let (conversation_id, _) =
        create_signal(Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap());

    let (model, set_model) = create_signal(String::from(MODEL));

    let db_messages = create_resource(
        || (),
        |_| async {
            get_conversation_messages(
                Uuid::from_str("1ec2aa50-b36d-4bf6-a9d8-ef5da43425bb").unwrap(),
            )
            .await
            .unwrap()
        },
    );

    let (messages, set_messages) = create_signal(Vec::<MessageModel>::new());
    let (user_prompt, set_user_prompt) = create_signal(String::new());

    let send_user_prompt = create_server_action::<AskAssistant>();
    let assistant_response_value = send_user_prompt.value();

    let (button_disabled, set_button_disabled) = create_signal(true);

    create_effect(move |_| {
        if user_prompt().len() == 0 || send_user_prompt.pending().get() {
            set_button_disabled(true)
        } else {
            set_button_disabled(false)
        };
    });

    create_effect(move |_| {
        if let Some(response) = assistant_response_value.get() {
            let assistant_response = response.unwrap();
            set_messages.update(|msgs| msgs.push(assistant_response));
        }
    });

    let bottom_of_chat_div = create_node_ref::<Div>();
    create_effect(move |_| {
        let _ = messages();
        if let Some(div) = bottom_of_chat_div.get() {
            // TODO: I need to scroll with options
            // https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView
            div.scroll_into_view();
        }
    });

    let user_prompt_textarea = create_node_ref::<Textarea>();

    view! {
        <div class="flex max-w-full flex-1 flex-col">
            <div class="relative h-full w-full transition-width flex flex-col overflow-hidden items-stretch flex-1">
                <div class="flex-1 overflow-hidden">
                    <div class="scroll-to-bottom--css-ikyem-79elbk h-full dark:bg-gray-800">
                        <div class="scroll-to-bottom--css-ikyem-1n7m0yu">
                            <div class="flex flex-col items-center text-sm bg-gray-800">
                                <Transition fallback=move || {
                                    view! {
                                        <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                            "Loading initial data..."
                                        </div>
                                    }
                                }>
                                    {if let Some(messages) = db_messages.get() {
                                        set_messages(messages);
                                    }}
                                    <div class="flex w-full items-center justify-center gap-1 border-b border-black/10 bg-gray-50 p-3 text-gray-500 dark:border-gray-900/50 dark:bg-gray-700 dark:text-gray-300">
                                        "Model: " <b>{model}</b>
                                    </div> <Messages messages=messages.into()/>
                                    <div class="w-full h-32 flex-shrink-0"></div>
                                    <div node_ref=bottom_of_chat_div></div>
                                </Transition>
                            </div>
                            <div class="flex flex-col items-center text-sm dark:bg-gray-800"></div>
                        </div>
                    </div>
                </div>
                <div class="absolute bottom-0 left-0 w-full border-t md:border-t-0 dark:border-white/20 md:border-transparent md:dark:border-transparent md:bg-vert-light-gradient bg-white dark:bg-gray-800 md:!bg-transparent dark:md:bg-vert-dark-gradient pt-2">
                    <form class="stretch mx-2 flex flex-row gap-3 last:mb-2 md:mx-4 md:last:mb-6 lg:mx-auto lg:max-w-2xl xl:max-w-3xl">

                        <div class="relative flex flex-col h-full flex-1 items-stretch md:flex-col">
                            // {errorMessage ? (
                            // <div class="mb-2 md:mb-0">
                            // <div class="h-full flex ml-1 md:w-full md:m-auto md:mb-2 gap-0 md:gap-2 justify-center">
                            // <span class="text-red-500 text-sm">{errorMessage}</span>
                            // </div>
                            // </div>
                            // ) : null}
                            <div class="flex flex-col w-full py-2 flex-grow md:py-3 md:pl-4 relative border border-black/10 bg-white dark:border-gray-900/50 dark:text-white dark:bg-gray-700 rounded-md shadow-[0_0_10px_rgba(0,0,0,0.10)] dark:shadow-[0_0_15px_rgba(0,0,0,0.10)]">
                                <textarea
                                    on:input=move |ev| {
                                        set_user_prompt(event_target_value(&ev));
                                    }

                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" && !ev.shift_key() {
                                            ev.prevent_default();
                                            let user_message = MessageModel::user(
                                                user_prompt(),
                                                conversation_id(),
                                            );
                                            let user_message_clone = user_message.clone();
                                            if user_message.content != "" {
                                                set_messages.update(|msgs| msgs.push(user_message_clone));
                                                send_user_prompt.dispatch(AskAssistant { user_message });
                                                set_user_prompt("".to_string());
                                            }
                                        }
                                    }

                                    type="text"
                                    placeholder="Message assistant..."
                                    prop:value=user_prompt
                                    node_ref=user_prompt_textarea
                                    // ref={textAreaRef} <- important for auto scrolling
                                    // tabIndex={0} <- no idea
                                    // data-id="root" <- no idea
                                    // style={{ <- no idea
                                    // height: "24px", <- no idea
                                    // maxHeight: "200px", <- no idea
                                    // overflowY: "hidden", <- no idea
                                    // }} <- no idea
                                    rows=1
                                    placeholder="Send a message..."
                                    class="m-0 w-full resize-none border-0 bg-transparent p-0 pr-7 focus:ring-0 focus-visible:ring-0 dark:bg-transparent pl-2 md:pl-0 h-[24px] max-h-[200px] overflow-y-hidden"
                                ></textarea>
                                <button
                                    class="absolute p-1 rounded-md bottom-1.5 md:bottom-2.5 bg-transparent disabled:bg-gray-500 right-1 md:right-2 disabled:opacity-40"
                                    on:click=move |ev| {
                                        ev.prevent_default();
                                        let user_message = MessageModel::user(
                                            user_prompt(),
                                            conversation_id(),
                                        );
                                        let user_message_clone = user_message.clone();
                                        if user_message.content != "" {
                                            set_messages.update(|msgs| msgs.push(user_message_clone));
                                            send_user_prompt.dispatch(AskAssistant { user_message });
                                            set_user_prompt("".to_string());
                                        }
                                    }

                                    disabled=button_disabled
                                >

                                    <Icon icon=icondata::LuSend class="h-4 w-4 mr-1 text-white "/>
                                </button>
                            </div>
                        </div>
                    </form>
                    <div class="px-3 pt-2 pb-3 text-center text-xs text-black/50 dark:text-white/50 md:px-4 md:pt-3 md:pb-6">
                        <span>Enjoy your self-hosted LokAI!</span>
                    </div>
                </div>
            </div>
        </div>
    }
}