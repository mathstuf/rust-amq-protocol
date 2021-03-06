use crate::{
    frame::AMQPFrame,
    protocol::{basic::gen_properties, *},
    types::{generation::*, *},
};
use cookie_factory::{
    combinator::slice,
    sequence::{pair, tuple},
};
use std::io::Write;

/// Serialize a frame in the given buffer
pub fn gen_frame<'a, W: Write + BackToTheBuffer + 'a>(
    frame: &'a AMQPFrame,
) -> impl SerializeFn<W> + 'a {
    move |x| match frame {
        AMQPFrame::ProtocolHeader => gen_protocol_header()(x),
        AMQPFrame::Heartbeat(_) => gen_heartbeat_frame()(x),
        AMQPFrame::Method(channel_id, method) => gen_method_frame(*channel_id, method)(x),
        AMQPFrame::Header(channel_id, class_id, header) => {
            gen_content_header_frame(*channel_id, *class_id, header.body_size, &header.properties)(
                x,
            )
        }
        AMQPFrame::Body(channel_id, data) => gen_content_body_frame(*channel_id, data)(x),
    }
}

fn gen_protocol_header<W: Write>() -> impl SerializeFn<W> {
    pair(
        slice(metadata::NAME.as_bytes()),
        slice(&[
            0,
            metadata::MAJOR_VERSION,
            metadata::MINOR_VERSION,
            metadata::REVISION,
        ]),
    )
}

fn gen_heartbeat_frame<W: Write>() -> impl SerializeFn<W> {
    slice(&[
        constants::FRAME_HEARTBEAT,
        0,
        0,
        0,
        0,
        0,
        0,
        constants::FRAME_END,
    ])
}

fn gen_method_frame<'a, W: Write + BackToTheBuffer + 'a>(
    channel_id: ShortUInt,
    class: &'a AMQPClass,
) -> impl SerializeFn<W> + 'a {
    tuple((
        gen_short_short_uint(constants::FRAME_METHOD),
        gen_id(channel_id),
        gen_with_len(gen_class(class)),
        gen_short_short_uint(constants::FRAME_END),
    ))
}

fn gen_content_header_frame<'a, W: Write + BackToTheBuffer + 'a>(
    channel_id: ShortUInt,
    class_id: ShortUInt,
    length: LongLongUInt,
    properties: &'a basic::AMQPProperties,
) -> impl SerializeFn<W> + 'a {
    tuple((
        gen_short_short_uint(constants::FRAME_HEADER),
        gen_id(channel_id),
        gen_with_len(tuple((
            gen_id(class_id),
            gen_short_uint(0 /* weight */),
            gen_long_long_uint(length),
            gen_properties(properties),
        ))),
        gen_short_short_uint(constants::FRAME_END),
    ))
}

fn gen_content_body_frame<'a, W: Write + 'a>(
    channel_id: ShortUInt,
    content: &'a [u8],
) -> impl SerializeFn<W> + 'a {
    tuple((
        gen_short_short_uint(constants::FRAME_BODY),
        gen_id(channel_id),
        gen_long_uint(content.len() as LongUInt),
        slice(content),
        gen_short_short_uint(constants::FRAME_END),
    ))
}
