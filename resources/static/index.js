// 光标默认可输入
$(document).ready(function (e) {
        $('#word').focus();
    }
);

// 查询mdx
function queryMdx(word) {
    $('#mdx-resp').html('查询中...');
    $.ajax({
        url: './query',
        type: 'POST',
        data: {'word': word},
        dataType: 'json',  // 确保返回的是JSON格式
        success: function (response) {
            if (response.data && response.data.length > 0) {
                const cards = response.data.map(item => {
                    // 清理内容中的特殊字符
                    const content = item.content.replace(/\u0000/g, '');
                    
                    return `
                        <div class="dict-card">
                            <div class="dict-header">${item.dict}</div>
                            <div class="dict-content">${content}</div>
                        </div>
                    `;
                }).join('');
                
                $('#mdx-resp').html(cards).show();
            } else {
                $('#mdx-resp').hide();
            }
        },
        error: function() {
            $('#mdx-resp').html('查询失败').show();
        }
    });
}

function postQuery() {
    let word = $('#word').val().trim();
    if (!validInput(word)) {
        return;
    }
    queryMdx(word);
}

// 特殊字符不查询
function validInput(word) {
    return word
        && word !== '.'
        && word !== '#'
        && word !== '?'
        && word !== '/';
}

// 监听回车键
$(document).keydown(function (e) {
    if (e.keyCode === 13) {
        postQuery();
    }
});

// 监听牛津8解释页面的外部单词链接
$(document).on('click', 'a', function (e) {
    console.log($(this).attr('href'));
    let href = $(this).attr('href');// '/cool'
    if (href.startsWith('/') && !href.startsWith('/#')) {
        $('#word').val(href.slice(1)) // 'cool'
        postQuery();
        e.preventDefault()
    }
});

// 捕获ctrl+L快捷键
$(window).bind('keyup keydown', function (e) {
    if ((e.ctrlKey || e.metaKey)
        && String.fromCharCode(e.which).toLowerCase() === 'l') {
        e.preventDefault();
        $('#word').val('').focus();
        scrollTo(0, 0);
    }
});

// 试试手气按钮
$(document).on('click', '#lucky-btn', function (e) {
    $.ajax({
        url: './lucky',
        type: 'GET',
        dataType: 'html',
        success: function (data) {
            if (data !== '') {
                $('#mdx-resp').html(data).show();
            } else {
                $('#mdx-resp').hide();
            }
            // $('#word').val(parserWordFromResp(data))
        }
    });
});

// 不同词典返回html不一样，无法通用
// function parserWordFromResp(data) {
//     let el = document.createElement('html');
//     el.innerHTML = data;
//     let top_g = el.getElementsByClassName("top-g")[0]
//     if (top_g == null) {
//         console.log("top-g is null");
//         return "";
//     }
//
//     return top_g.firstElementChild.innerHTML.split('·').join('')
//
// }